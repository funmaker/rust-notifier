var connection

HTMLElement.prototype.appendFirst=function(childNode){
    if(this.firstChild)this.insertBefore(childNode,this.firstChild);
    else this.appendChild(childNode);
};

window.addEventListener("load", function () {
	connection = new WebSocket("ws://ks.sebi.moe:9039")
	connection.onopen = function () {
		console.log("Connection opened")
		document.getElementById("form").onsubmit = function (event) {
			var msg = document.getElementById("msg")
			if (msg.value){
				connection.send(msg.value)
			}
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
				if (typeof value === "string" && value.length > 500) {
					return value.substring(0,500)+"...";
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
        "fetch":
`{
    "command": "fetch",
    "feeds": [
        "*"
    ],
    "flat": false
}`,
        "list":
`{
    "command": "list"
}`,
        "add":
`{
    "command": "add",
    "feedName": "funmaker-rss-mikufan",
    "entry": {
        "color": "#1B94D1",
        "provider": "rss",
        "providerData": "http://feeds.feedburner.com/Mikufancom?format=xml"
    }
}`,
        "remove":
`{
    "command": "remove",
    "feedName": "funmaker-rss-mikufan"
}`,
	}
	
	for (var name in templates) {
		(function (name){
			document.getElementById(name).onclick = function(){
				document.getElementById("msg").value = templates[name]
			}
		})(name)
	}
})
