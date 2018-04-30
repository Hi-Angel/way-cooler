//! TODO Fill in

use std::default::Default;
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, Mutex, MutexGuard, TryLockResult};

use cairo::{Format, ImageSurface};
use glib::translate::ToGlibPtr;
use rlua::{self, AnyUserData, LightUserData, Lua, Table, ToLua, UserData, UserDataMethods, Value};
use wlroots::{Area, Origin, Size};

use super::class::{self, Class};
use super::object::{self, Object, Objectable};
use super::property::Property;

#[derive(Clone, Debug)]
pub struct SharedImage {
    image: Arc<Mutex<Option<ImageSurface>>>
}

impl SharedImage {
    /// Makes a new shared image out of an ImageSurface.
    pub fn new<O>(image: O) -> Self
        where O: Into<Option<ImageSurface>>
    {
        let image = Arc::new(Mutex::new(image.into()));
        SharedImage { image }
    }

    /// Attempts to get a mutable reference to the underlying image.
    ///
    /// If the lock could not be acquired, returns an Error.
    pub fn image(&self) -> TryLockResult<MutexGuard<Option<ImageSurface>>> {
        self.image.try_lock()
    }
}

#[derive(Clone, Debug)]
pub struct DrawableState {
    pub surface: SharedImage,
    geo: Area,
    // TODO Use this to determine whether we draw this or not
    refreshed: bool
}

pub struct Drawable<'lua>(Object<'lua>);

impl_objectable!(Drawable, DrawableState);

impl Default for DrawableState {
    fn default() -> Self {
        DrawableState { surface: SharedImage::new(None),
                        geo: Area::default(),
                        refreshed: false }
    }
}

impl<'lua> Drawable<'lua> {
    pub fn new(lua: &Lua) -> rlua::Result<Object> {
        let class = class::class_setup(lua, "drawable")?;
        let builder = Drawable::allocate(lua, class)?;
        // TODO Do properly
        let table = lua.create_table()?;
        table.set("geometry", lua.create_function(geometry)?)?;
        table.set("refresh", lua.create_function(refresh)?)?;
        Ok(builder.add_to_meta(table)?.build())
    }

    pub fn get_geometry(&self) -> rlua::Result<Area> {
        let drawable = self.state()?;
        Ok(drawable.geo)
    }

    pub fn get_surface(&self) -> rlua::Result<Value<'lua>> {
        let drawable = self.state()?;
        let lock = drawable.surface.image();
        Ok(match lock {
            Err(_) => Value::Nil,
            Ok(ref none) if none.is_none() => Value::Nil,
            Ok(ref image) => {
                let stash = image.to_glib_none();
                let ptr = stash.0;
                // NOTE
                // We bump the reference count because now Lua has a reference which
                // it manages via LGI.
                //
                // If there's a bug, worst case scenario there's a memory leak.
                unsafe {
                    ::cairo_sys::cairo_surface_reference(ptr);
                }
                Value::LightUserData(LightUserData(ptr as _))
            }
        })
    }

    /// Sets the geometry, and allocates a new surface.
    pub fn set_geometry(&mut self, geometry: Area) -> rlua::Result<()> {
        use rlua::Error::RuntimeError;
        let mut drawable = self.get_object_mut()?;
        let size_changed = drawable.geo != geometry;
        drawable.geo = geometry;
        if size_changed {
            drawable.refreshed = false;
            let mut image = drawable.surface.image().expect("Could not lock image");
            *image = None;
            let size: Size = geometry.size;
            if size.width > 0 && size.height > 0 {
                *image = Some(ImageSurface::create(Format::ARgb32,
                                                        size.width,
                                                        size.height)
                    .map_err(|err| RuntimeError(format!("Could not allocate {:?}", err)))?);
                // TODO emity property::surface
            }
        }
        Ok(())
    }

    /// Signals that the drawable's surface was updated.
    pub fn refresh(&mut self) -> rlua::Result<()> {
        let mut drawable = self.get_object_mut()?;
        drawable.refreshed = true;
        Ok(())
    }
}

impl Display for DrawableState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Drawable: {:p}", self)
    }
}

impl<'lua> ToLua<'lua> for Drawable<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for DrawableState {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        object::default_add_methods(methods);
    }
}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    Class::builder(lua, "drawable", None)?
        .method("geometry".into(), lua.create_function(geometry)?)?
        .property(Property::new("surface".into(),
                                None,
                                Some(lua.create_function(get_surface)?),
                                None))?
        .save_class("drawable")?
        .build()
}

fn get_surface<'lua>(_: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<Value<'lua>> {
    let drawable = Drawable::cast(obj.into())?;
    drawable.get_surface()
}

fn geometry<'lua>(lua: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<Table<'lua>> {
    let drawable = Drawable::cast(obj.into())?;
    let geometry = drawable.get_geometry()?;
    let Origin { x, y } = geometry.origin;
    let Size { width, height } = geometry.size;
    let table = lua.create_table()?;
    table.set("x", x)?;
    table.set("y", y)?;
    table.set("width", width)?;
    table.set("height", height)?;
    Ok(table)
}

fn refresh<'lua>(_: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<()> {
    Drawable::cast(obj.into())?.refresh()
}
