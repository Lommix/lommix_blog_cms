// Using js in the intended way, small animations and transitions.


// @description Toggle an element
// @param {string} id
function toggle(id) {
	const element = document.getElementById(id);
	element.classList.toggle("hidden");
}

// @description slide in a element
// @param {string} id
// @param {number} pixel
function slide_down(id, pixel) {
	const element = document.getElementById(id);
	if (parseInt(element.style.height) > 0) {
		element.style.height = "0px";
	} else {
		element.style.height = `${pixel}px`;
	}
}

// @description import a wasm runtime
// @param {string} wasm_path
// @param {string} script_path
// @param {string} canvas_id
// @param {number} height
async function run_wasm(wasm_path, script_path, canvas_id, height) {
	const script = await import(script_path);
	let canvas = document.getElementById(canvas_id.replace("#", ""));
	const max_width = canvas.parentNode.clientWidth;
	const loading_screen = document.createElement("div");

	loading_screen.innerHTML = `<div id="loading-screen" class="bg-black block flex justify-center items-center" style="height: ${height}px; width: ${max_width}px;">
		<div class="text-white text-4xl flex flex-row spacing-x-5"><div class="spinner"></div>Loading</div>
	</div>`;

	canvas.parentNode.appendChild(loading_screen);
	loading_screen.scrollIntoView({
		behavior: 'smooth',
	});

	fetch(wasm_path)
		.then((res) => res.arrayBuffer())
		.then(async (bytes) => {
			await script.initSync(bytes);
			await script.init();
			document.getElementById("loading-screen").remove();
			await script.run(canvas_id, max_width, height);
		});
}
