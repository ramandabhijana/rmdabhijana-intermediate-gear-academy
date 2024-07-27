macro_rules! reply {
    ($payload:expr) => {{
        use gstd::{msg, Encode};

        fn reply_wrapper<E: Encode>(payload: E) {
            msg::reply(payload, 0).expect("Error in sending reply");
        }
        reply_wrapper($payload)
    }};
}
