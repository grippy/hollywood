// Testing the broker looks like this:
// 1. Run the broker in a thread
// 2. Main thread should...
//  3. Write messages to nats queue
//  4. Broker should read message from nats
//    and move them to the internal actor mailbox
// 5. Main thread should read them from actor mailbox
//    and assert the messages are correct
// 6. Main thread should shutdown broker thread

fn setup() {
    // instantiate: nats client
    // instantiate: broker
}

#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}
