var connection

HTMLElement.prototype.appendFirst=function(childNode){
    if(this.firstChild)this.insertBefore(childNode,this.firstChild);
    else this.appendChild(childNode);
};

window.addEventListener("load", function () {
	connection = new WebSocket("ws://127.0.0.1:9039")
	connection.onopen = function () {
		console.log("Connection opened")
		document.getElementById("form").onsubmit = function (event) {
			var msg = document.getElementById("msg")
			if (msg.value){
				connection.send(msg.value)
			}
			msg.value = ""
			event.preventDefault()
		}
	}
	connection.onclose = function (event) {
		var reason = "";
		switch (event.code)
			{
				case 1001:
					reason = "Endpoint going away.";
					break;
				case 1002:
					reason = "Protocol error.";
					break;
				case 1003:
					reason = "Unsupported message.";
					break;
				case 1005:
					reason = "No status.";
					break;
				case 1006:
					reason = "Abnormal disconnection.";
					break;
				case 1009:
					reason = "Data frame too large.";
					break;
				default:
					reason = "Unknown Error";
			}
		alert("Connection closed, " + event.code + " - " + reason)
	}
	connection.onerror = function (event) {
		console.error("Connection error")
	}
	connection.onmessage = function (event) {
		var pre = document.createElement("pre")
		pre.textContent = event.data
		try{
			function replacer(key, value) {
				if (typeof value === "string" && value.length > 50) {
					return value.substring(0,50)+"...";
				}
				return value;
			}
			pre.textContent = JSON.stringify(JSON.parse(event.data), replacer, 2)
		}catch(e){
			console.error("Parsing Error: "+e.name+" "+e.message)
		}
		document.getElementById("log").appendFirst(pre)
	}
	
	templates = {
"handshake":
'{\n\
  "ID":0,\n\
  "data":{\n\
    "name":"Janusz",\n\
	"secret":"abc"\n\
  }\n\
}',
"respawn":
'{\n\
  "ID":8,\n\
  "data":{}\n\
}',
"split":
'{\n\
  "ID":5,\n\
  "data":{\n\
    "direction":[0,1]\n\
  }\n\
}',
"update":
'{\n\
  "ID":4,\n\
  "data":{\n\
    "dots":[\n\
      {\n\
        "id":0,\n\
        "pos":[0,1],\n\
        "dir":[0,10]\n\
      }\n\
    ]\n\
  }\n\
}',
	}
	
	for (var name in templates) {
		(function (name){
			document.getElementById(name).onclick = function(){
				document.getElementById("msg").value = templates[name]
			}
		})(name)
	}
})
