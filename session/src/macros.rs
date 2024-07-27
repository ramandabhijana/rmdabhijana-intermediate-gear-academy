macro_rules! reply {
    ($payload:expr) => {{
        use gstd::{msg, Encode};

        fn reply_wrapper<E: Encode>(payload: E) {
            msg::reply(payload, 0).expect("Error in sending reply");
        }
        reply_wrapper($payload)
    }};
}

macro_rules! create_inner_state {
    ($ident:ident, $type:ty) => {
        static mut $ident: Option<$type> = None;

        pub unsafe fn init_inner_state(val: $type) {
            $ident = Some(val);
        }

        pub fn get_inner_state_mut() -> &'static mut $type {
            unsafe { $ident.as_mut().expect("State is not initialized") }
        }

        pub fn get_inner_state() -> $type {
            unsafe { $ident.take().expect("State is not initialized") }
        }
    };
}
