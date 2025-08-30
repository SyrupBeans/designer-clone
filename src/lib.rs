use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "trace_clone")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[cfg(feature = "trace_clone")]
mod event {
    use super::*;

    pub(crate) struct Event<T, R = ()> {
        callback: Rc<RefCell<dyn FnMut(&mut T) -> R>>,
    }

    impl<T, R> Debug for Event<T, R> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Event({:p})", self.callback.as_ref() as *const _)
        }
    }

    impl<T, R> Clone for Event<T, R> {
        fn clone(&self) -> Self {
            Self {
                callback: Rc::clone(&self.callback),
            }
        }
    }

    impl<T, R> Event<T, R> {
        pub(crate) fn new(callback: impl (for<'a> FnMut(&'a mut T) -> R) + 'static) -> Self {
            Self {
                callback: Rc::new(RefCell::new(callback)),
            }
        }

        pub(crate) fn fire(&self, arg: &mut T) -> R {
            self.callback.borrow_mut()(arg)
        }
    }
}

#[cfg(feature = "trace_clone")]
use event::*;

pub struct Tr<T> {
    value: T,

    #[cfg(feature = "trace_clone")]
    on_cloning: Event<T, bool>,

    #[cfg(feature = "trace_clone")]
    on_cloned: Event<T>,

    #[cfg(feature = "trace_clone")]
    suspended: Cell<bool>,
}

impl<T> Deref for Tr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Tr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> From<T> for Tr<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Debug> Debug for Tr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "trace_clone")]
        {
            write!(
                f,
                "Tr({:?}, <suspended: {}>, <on_cloning: {:?}>, <on_cloned: {:?}>)",
                self.value,
                self.suspended.get(),
                self.on_cloning,
                self.on_cloned,
            )
        }

        #[cfg(not(feature = "trace_clone"))]
        {
            write!(f, "Tr({:?})", self.value)
        }
    }
}

impl<T: Default> Default for Tr<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: PartialEq> PartialEq for Tr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for Tr<T> {}

impl<T: PartialOrd> PartialOrd for Tr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord> Ord for Tr<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[cfg_attr(not(feature = "trace_clone"), allow(unused_variables))]
impl<T> Tr<T> {
    pub fn with_closure(
        value: T,
        on_cloning: impl (for<'a> FnMut(&'a mut T) -> bool) + 'static,
        on_cloned: impl (for<'a> FnMut(&'a mut T)) + 'static,
    ) -> Self {
        #[cfg(feature = "trace_clone")]
        {
            Self {
                value,
                on_cloning: Event::new(on_cloning),
                on_cloned: Event::new(on_cloned),
                suspended: Cell::new(false),
            }
        }

        #[cfg(not(feature = "trace_clone"))]
        {
            Self { value }
        }
    }

    pub fn new(value: T) -> Self {
        let this = Self::with_closure(
            value,
            |_| false,
            |_| {
                dbg!("Unknown clone detected.");
            },
        );
        #[cfg(feature = "trace_clone")]
        {
            Tr::suspend(&this);
        }
        this
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    #[cfg(feature = "trace_clone")]
    pub fn suspend(this: &Self) {
        this.suspended.set(true);
    }

    #[cfg(feature = "trace_clone")]
    pub fn resume(this: &Self) {
        this.suspended.set(false);
    }
}

pub trait Traced: Clone {
    fn traced(self) -> Tr<Self> {
        Tr::new(self)
    }
}

impl<T: Clone> Traced for T {}

impl<T: Clone> Clone for Tr<T> {
    fn clone(&self) -> Self {
        #[cfg(not(feature = "trace_clone"))]
        {
            let value = self.value.clone();
            Self { value }
        }

        #[cfg(feature = "trace_clone")]
        {
            let mut value = self.value.clone();
            let sus = self.on_cloning.fire(&mut value) || self.suspended.get();
            self.on_cloned.fire(&mut value);
            if !sus {
                panic!("Unknown clone not forgiven.")
            }

            Self {
                value,
                on_cloning: self.on_cloning.clone(),
                on_cloned: self.on_cloned.clone(),
                suspended: false.into(),
            }
        }
    }
}

pub trait CloneSilent: Clone {
    fn clone_silent(&self) -> Self;
}

impl<T: Clone> CloneSilent for Tr<T> {
    fn clone_silent(&self) -> Self {
        #[cfg(feature = "trace_clone")]
        Self::suspend(self);

        let clone = self.clone();

        #[cfg(feature = "trace_clone")]
        Self::resume(self);
        
        clone
    }
}

pub struct Tag<T, V> {
    pub value: V,
    pub tag: T,
}

impl<T: Debug, V: Debug> Debug for Tag<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tag({:?}, {:?})", self.value, self.tag)
    }
}

impl<T: Clone, V: Clone> Clone for Tag<T, V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            tag: self.tag.clone(),
        }
    }
}

impl<T, V> Deref for Tag<T, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, V> DerefMut for Tag<T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, V> Tag<T, V> {
    pub fn tag(this: &Self) -> &T {
        &this.tag
    }

    pub fn tag_mut(this: &mut Self) -> &mut T {
        &mut this.tag
    }
}

pub trait Tagged<T>: Sized {
    fn tagged(self, tag: T) -> Tag<T, Self> {
        Tag { value: self, tag }
    }
}

impl<T: Sized, V> Tagged<T> for V {}

#[cfg(all(test, feature = "trace_clone"))]
mod test;
