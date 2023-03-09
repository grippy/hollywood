use tokio_test;

macro_rules! block_on {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}
