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
// @param {number} width
// @param {number} height
async function run_wasm(wasm_path, script_path, canvas_id, width, height) {
	const script = await import("/static/media/1/boids-quadtree.js");
	fetch("/static/media/1/boids-quadtree_bg.wasm")
		.then((res) => res.arrayBuffer())
		.then(async (bytes) => {
			await script.initSync(bytes);
			await script.init();
			await script.run('#boids', 900, 900);
		});
}
