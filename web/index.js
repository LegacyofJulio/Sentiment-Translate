async function sendData() {
  const input_prompt = document.getElementById('userInput').value;
  const response_div = document.getElementById('response');
  try {
    const response = await fetch('http://127.0.0.1:3000/gen', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ input: input_prompt })
    });
    const data = await response.json();
    const data_to_be_send = data['response'];
    response_div.innerHTML = '<h3>Response</h3>' + data_to_be_send;

  } catch (error) {
    console.log(error);
  }
}
