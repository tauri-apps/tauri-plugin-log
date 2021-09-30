<script lang="ts">
	import * as log from 'tauri-plugin-log-api'

	let message = ''
	let level = 'info'

	log.attachConsole()

	function send () {
      log[level](message)
    }
</script>

<div>
	<select id="level" bind:value={level}>
		<option value="trace">Trace</option>
		<option value="debug">Debug</option>
		<option value="info">Info</option>
		<option value="warn">Warn</option>
		<option value="error">Error</option>
	</select>
	<input id="message" bind:value={message}>
	<button id="send" on:click="{send}">Send</button>
</div>
