/**
 * Log a message to the console
 *
 * @name log_me()
 * @param {string} str - The string to log.
 * @returns {void}
 */
function log_me(str) {
  console.log("Log: ", str);
}

/**
 * Display the port number in the HTML
 *
 * @returns {void}
 */
function displayPort() {
  const port = window.location.port;
  const portInfo = document.getElementById("port-info");
  if (port) {
    portInfo.textContent = `Server is running on port: ${port}`;
  } else {
    portInfo.textContent = "Server is running on the default port.";
  }
}
