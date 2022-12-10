/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![macro_use]

macro_rules! impl_builtin_traits_inner {
    ( Default for $Type:ty => $gd_method:ident ) => {
        impl Default for $Type {
            #[inline]
            fn default() -> Self {
                unsafe {
                    let mut gd_val = sys::$GdType::default();
                    ::godot_ffi::builtin_fn!($gd_method)(&mut gd_val);
                    <$Type>::from_sys(gd_val)
                }
            }
        }
    };

    ( Clone for $Type:ty => $gd_method:ident ) => {
        impl Clone for $Type {
            #[inline]
            fn clone(&self) -> Self {
                unsafe {
                    Self::from_sys_init(|self_ptr| {
                        let ctor = ::godot_ffi::builtin_fn!($gd_method);
                        let mut args = [self.sys_const()];
                        ctor(self_ptr, args.as_mut_ptr());
                    })
                }
            }
        }
    };

    ( Drop for $Type:ty => $gd_method:ident ) => {
        impl Drop for $Type {
            #[inline]
            fn drop(&mut self) {
                unsafe {
                    let destructor = ::godot_ffi::builtin_fn!($gd_method @1);
            	    destructor(self.sys_mut());
                }
            }
        }
    };

    ( PartialEq for $Type:ty => $gd_method:ident ) => {
        impl PartialEq for $Type {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                unsafe {
                    let mut result = false;
                    ::godot_ffi::builtin_call! {
                        $gd_method(self.sys(), other.sys(), result.sys_mut())
                    };
                    result
                }
            }
        }
    };

    ( Eq for $Type:ty => $gd_method:ident ) => {
		impl_builtin_traits_inner!(PartialEq for $Type => $gd_method);
        impl Eq for $Type {}
    };

    ( PartialOrd for $Type:ty => $gd_method:ident ) => {
        impl PartialOrd for $Type {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                let op_less = |lhs, rhs| unsafe {
                    let mut result = false;
                    ::godot_ffi::builtin_call! {
                        $gd_method(lhs, rhs, result.sys_mut())
                    };
                    result
                };

                if op_less(self.sys(), other.sys()) {
                    Some(std::cmp::Ordering::Less)
                } else if op_less(other.sys(), self.sys()) {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    Some(std::cmp::Ordering::Equal)
                }
            }
        }
    };

    ( Ord for $Type:ty => $gd_method:ident ) => {
        impl_builtin_traits_inner!(PartialOrd for $Type => $gd_method);
        impl Ord for $Type {
            #[inline]
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                PartialOrd::partial_cmp(self, other).expect("PartialOrd::partial_cmp")
            }
        }
    };
}

macro_rules! impl_builtin_traits {
    (
        for $Type:ty {
            $( $Trait:ident => $gd_method:ident; )*
        }
    ) => (
        $(
            impl_builtin_traits_inner! {
                $Trait for $Type => $gd_method
            }
        )*
    )
}

macro_rules! impl_builtin_stub {
    ($Class:ident, $OpaqueTy:ident) => {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct $Class {
            opaque: sys::types::$OpaqueTy,
        }

        impl $Class {
            fn from_opaque(opaque: sys::types::$OpaqueTy) -> Self {
                Self { opaque }
            }
        }

        impl GodotFfi for $Class {
            ffi_methods! { type sys::GDNativeTypePtr = *mut Opaque; .. }
        }
    };
}

macro_rules! impl_builtin_froms {
    ($To:ty; $($From:ty => $from_fn:ident),* $(,)?) => {
        $(impl From<&$From> for $To {
            fn from(other: &$From) -> Self {
                unsafe {
                    Self::from_sys_init(|ptr| {
                        let mut args = [other.sys_const()];
                        ::godot_ffi::builtin_call! {
                            $from_fn(ptr, args.as_mut_ptr())
                        }
                    })
                }
            }
        })*
    };
}
