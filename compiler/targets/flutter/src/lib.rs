#![no_std]

extern crate alloc;

#[global_allocator]
static GLOBAL: GlobalDlmalloc = GlobalDlmalloc;

use alloc::{boxed::Box, ffi::CString};
use dlmalloc::GlobalDlmalloc;
use dropin_compiler_recipes::ir::Model;
use dropin_target_macros::combine;
use prost::Message;

use crate::{
  gen::Gen,
  imports::{Imports, ImportsState},
  listeners::{Listeners, ListenersState},
  objects_getter::{ObjectGetter, ObjectGetterState},
};

trait Stage {
  fn ir(&self) -> &Model;
}

trait Stated<S> {
  fn state(&self) -> &S;
}

impl Stage for Model {
  fn ir(&self) -> &Model {
    self
  }
}

mod absolute_getters;
mod gen;
mod imports;
mod listeners;
mod objects_getter;
mod setters;

#[combine]
struct Combine<'a>(
  #[state(ObjectGetterState<'a>)] ObjectGetter<'a>,
  #[state(ListenersState<'a>)] Listeners<'a>,
  #[state(ImportsState)] Imports<'a>,
);

#[no_mangle]
pub fn codegen(protobuf: *mut [u8]) -> CString {
  let protobuf = unsafe { Box::from_raw(protobuf) };
  let component = Model::decode(protobuf.as_ref()).unwrap();
  let objects_getter = ObjectGetter::new(&component);
  let listeners = Listeners::new(&component);
  let imports = Imports::new(&component);
  let combine = Combine(objects_getter, listeners, imports);
  let gen = Gen::new(&combine);
  CString::new(gen.gen().unwrap()).unwrap()
}

// #[cfg(debug_assertions)]
// use lazy_static::lazy_static;
// #[cfg(debug_assertions)]
// use wasi::cli::stdout::OutputStream;

// #[cfg(debug_assertions)]
// lazy_static! {
//   static ref STDOUT: OutputStream = wasi::cli::stdout::get_stdout();
// }

// #[cfg(debug_assertions)]
// struct Printer;

// #[cfg(debug_assertions)]
// impl core::fmt::Write for Printer {
//   fn write_str(&mut self, s: &str) -> core::fmt::Result {
//     crate::STDOUT.write(s.as_bytes()).unwrap();
//     Ok(())
//   }
// }
