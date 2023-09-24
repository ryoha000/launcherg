import Toastify from "toastify-js";

export const showInfoToast = (text: string, duration = 3000) => {
  Toastify({
    text,
    duration: duration,
    gravity: "bottom", // `top` or `bottom`
    position: "right", // `left`, `center` or `right`
    stopOnFocus: true, // Prevents dismissing of toast on hover
    style: {
      background: "#22272e",
      border: "1px solid #444c56",
      "border-radius": "0.5rem",
    },
  }).showToast();
};

export const showErrorToast = (text: string) => {
  Toastify({
    text,
    duration: 5000,
    gravity: "bottom", // `top` or `bottom`
    position: "right", // `left`, `center` or `right`
    stopOnFocus: true, // Prevents dismissing of toast on hover
    style: {
      background: "#EA4E60",
      border: "1px solid #EA4E60",
      color: "#FFFFFF",
      "border-radius": "0.5rem",
    },
  }).showToast();
};
