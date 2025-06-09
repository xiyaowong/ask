mod ai;
mod command;
mod settings;

pub use ai::*;
pub use command::*;
pub use settings::*;

#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => {
      #[cfg(debug_assertions)]
      {
        println!("======== DEBUG START ========");
        println!($($arg)*);
        println!("======== DEBUG END =========");
      }
    };
}
