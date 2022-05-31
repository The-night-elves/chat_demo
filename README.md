# chat_demo
rust chat demo

## run server
` cargo run --example server `

## run grpc client
` cargo run --example grpc-client --features="gui"`

## run quic client
` cargo run --example quic-client --features="gui"`

1. login with username
2. subscribe to topic
3. send message to topic
4. topic boadcast message to subscribed users