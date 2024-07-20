let socket = new WebSocket("ws://localhost:3000");

socket.addEventListener("message", (event) => {
    console.log("Message from server ", event.data);
  });

socket.send("Hello from client!")