async function fetchFileList() {
  try {
    const res = await fetch("http://localhost:3000/files");
    if (!res.ok) throw new Error("Failed to fetch");

    const data: { 0: string[] } = await res.json(); // FileList is a tuple struct
    const list = data[0];

    const ul = document.getElementById("file-list") as HTMLUListElement;
    ul.innerHTML = ""; // Clear any previous entries

    list.forEach(file => {
      const li = document.createElement("li");
      li.textContent = file;
      ul.appendChild(li);
    });
  } catch (err) {
    console.error(err);
  }
}

window.addEventListener("DOMContentLoaded", fetchFileList);