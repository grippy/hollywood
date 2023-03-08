// Flow:
// 1. Launch docker-compose
// 2. Build examples in rust container
// 3. Run examples in "test" mode
// 4. Send messages to example actors
// 5. Assert responses
// 6. Trigger shutting down

#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}
