use std::{
    cell::{self, RefCell, UnsafeCell},
    error::Error,
    fmt,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use parking_lot::{Once, ReentrantMutex, ReentrantMutexGuard};

#[derive(Debug)]
pub struct BorrowFail;

impl fmt::Display for BorrowFail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "failed to borrow global value twice in same thread") }
}

impl Error for BorrowFail {}

type InnerPointer<T> = Arc<ReentrantMutex<RefCell<T>>>;

pub struct Global<T>(Immutable<InnerPointer<T>>);

unsafe impl<T: Send> Sync for Global<T> {}
unsafe impl<T: Send> Send for Global<T> {}

impl<T> Global<T> {
    pub const fn new() -> Self { Self(Immutable::new()) }
}

impl<T: Default + 'static> Global<T> {
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&*self.lock().expect("Couldn't immutably access global variable"))
    }

    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut *self.lock_mut().expect("Couldn't mutably access global variable"))
    }

    pub fn lock(&self) -> Result<GlobalGuard<T>, BorrowFail> {
        let mutex: Arc<_> = Arc::clone(&*self.0);
        let mutex_ptr = &*mutex as *const ReentrantMutex<RefCell<T>>;

        let mutex_guard = unsafe { (*mutex_ptr).lock() };
        let mutex_guard_ptr = &*mutex_guard as *const RefCell<T>;

        let ref_cell_guard = unsafe { (*mutex_guard_ptr).try_borrow().map_err(|_| BorrowFail)? };

        Ok(GlobalGuard {
            mutex: ManuallyDrop::new(mutex),
            mutex_guard: ManuallyDrop::new(mutex_guard),
            ref_cell_guard: ManuallyDrop::new(ref_cell_guard),
        })
    }

    pub fn lock_mut(&self) -> Result<GlobalGuardMut<T>, BorrowFail> {
        let mutex: Arc<_> = Arc::clone(&*self.0);
        let mutex_ptr = &*mutex as *const ReentrantMutex<RefCell<T>>;

        let mutex_guard = unsafe { (*mutex_ptr).lock() };
        let mutex_guard_ptr = &*mutex_guard as *const RefCell<T>;

        let ref_cell_guard = unsafe { (*mutex_guard_ptr).try_borrow_mut().map_err(|_| BorrowFail)? };

        Ok(GlobalGuardMut {
            mutex: ManuallyDrop::new(mutex),
            mutex_guard: ManuallyDrop::new(mutex_guard),
            ref_cell_guard: ManuallyDrop::new(ref_cell_guard),
        })
    }

    pub fn force_init(&self) { self.0.ensure_exists(); }
}

pub struct GlobalGuardMut<T: 'static> {
    mutex: ManuallyDrop<Arc<ReentrantMutex<RefCell<T>>>>,
    mutex_guard: ManuallyDrop<ReentrantMutexGuard<'static, RefCell<T>>>,
    ref_cell_guard: ManuallyDrop<cell::RefMut<'static, T>>,
}

impl<T: 'static> Drop for GlobalGuardMut<T> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.ref_cell_guard);
            ManuallyDrop::drop(&mut self.mutex_guard);
            ManuallyDrop::drop(&mut self.mutex);
        }
    }
}

impl<T: 'static> Deref for GlobalGuardMut<T> {
    type Target = T;

    fn deref(&self) -> &T { &*self.ref_cell_guard }
}

impl<T: 'static> DerefMut for GlobalGuardMut<T> {
    fn deref_mut(&mut self) -> &mut T { &mut *self.ref_cell_guard }
}

pub struct GlobalGuard<T: 'static> {
    mutex: ManuallyDrop<Arc<ReentrantMutex<RefCell<T>>>>,
    mutex_guard: ManuallyDrop<ReentrantMutexGuard<'static, RefCell<T>>>,
    ref_cell_guard: ManuallyDrop<cell::Ref<'static, T>>,
}

impl<T: 'static> Drop for GlobalGuard<T> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.ref_cell_guard);
            ManuallyDrop::drop(&mut self.mutex_guard);
            ManuallyDrop::drop(&mut self.mutex);
        }
    }
}

impl<T: 'static> Deref for GlobalGuard<T> {
    type Target = T;

    fn deref(&self) -> &T { &*self.ref_cell_guard }
}

pub struct Immutable<T> {
    once: Once,
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Drop for Immutable<T> {
    fn drop(&mut self) {
        if let parking_lot::OnceState::Done = self.once.state() {
            unsafe {
                std::ptr::drop_in_place((*self.inner.get()).as_mut_ptr());
            }
        }
    }
}

unsafe impl<T: Send> Send for Immutable<T> {}
unsafe impl<T: Sync> Sync for Immutable<T> {}

impl<T: Default> Immutable<T> {
    fn ensure_exists(&self) {
        self.once.call_once(|| unsafe {
            *self.inner.get() = MaybeUninit::new(T::default());
        });
    }

    pub fn force_init(&self) { self.ensure_exists(); }
}

impl<T> Immutable<T> {
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl<T: Default> Deref for Immutable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ensure_exists();
        unsafe { &*(*self.inner.get()).as_ptr() }
    }
}
