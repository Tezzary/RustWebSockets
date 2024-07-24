let socket = new WebSocket("ws://localhost:3000");

socket.addEventListener("message", (event) => {
    console.log("Message from server ", event.data);
  });
function tick(){
    try {
        socket.send("Hello from client");
        console.log("Message sent");
    } catch (error) {
        console.log("Error in sending message", error);
    }
    setTimeout(tick, 1000);
}
tick();