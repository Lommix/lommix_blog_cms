// @param {string} id
function toggle(id) {
  const element = document.getElementById(id);
  element.classList.toggle("hidden");
}

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
