let socket = null;

document.getElementById("btn_send").onclick = send_message;

document.getElementById("login").addEventListener("click", login);
document.getElementById("btn_sub").addEventListener("click", subscription);

ws_open()

function ws_open() {
    socket = new WebSocket("ws://localhost:8080/ws");

    socket.onopen = function (e) {
        console.log("ws open", e);

        socket.onclose = function (e) {
            console.log("ws close", e);
        }

        socket.onmessage = function (e) {
            console.log("ws message", e);
            let msg = JSON.parse(e.data);
            console.log(msg);
            append_messages(msg)
        }
    };
}

function send_message() {
    let msg = {
        topic: get_topic(),
        message: { send_message: document.getElementById("input_message").value },
    };
    let data = JSON.stringify(msg);

    socket.send(data);
}

// show message to list
function append_messages(msg) {
    // <tr>
    //   <td>Shining Star</td>
    //   <td>Earth, Wind, and Fire</td>
    //   <td>1975</td>
    // </tr>

    // append child
    const child = `
    <tr>
    <td>${msg.topic}</td>
    <td>${msg.sequence}</td>
    <td>${msg.message}</td>
  </tr>`

    document.getElementById("table_messages").insertAdjacentHTML("beforeend", child);
}

// subscribe topic
function subscription() {
    // {"topic":"room1","message":{"join_room":{}}}
    let msg = {
        topic: get_topic(),
        message: { join_room: {} },
    };
    let data = JSON.stringify(msg);
    console.log(data);

    socket.send(data);
}

function get_topic() {
    return document.getElementById("subscription").value;
}


function login() {
    // get element by id name and value
    let name = document.getElementById("name").value;
    // clean string space
    name = name.trim();
    if (name.length === 0) {
        alert("please input your name");
        return;
    }

    let msg = {
        topic: "",
        message: { login: { name } },
    };
    let data = JSON.stringify(msg);
    console.log(data);

    socket.send(data);
}

