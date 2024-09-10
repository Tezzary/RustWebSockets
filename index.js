let socket = new WebSocket("ws://127.0.0.1:3000");

socket.onopen = () => {
    console.log("Connection established");
    for (let i = 0; i < 10; i++) {
        socket.send("Hello Server");
    }
}

socket.onmessage = (event) => {
    console.log(event.data);
}