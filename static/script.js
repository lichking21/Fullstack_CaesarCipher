document.getElementById("encrypt_btn").addEventListener("click", () => {

    send_encryption(true);
});
document.getElementById("decrypt_btn").addEventListener("click", () => {

    send_encryption(false);
});
function send_encryption(encrypt) {

    const filePath = document.getElementById("file_input"); 
    // file variable -  contains loaded file
    const file = filePath.files[0];

    // check if file was loaded
    if (!file) {

        console.error("File is not selected!");
        return;
    }

    // file reading
    const reader = new FileReader();
    // onload - event that will work when file is fully readed
    reader.onload = function(event) {

        // contains the text content of the file
        const fileContent = event.target.result;

        // fileInfo struct
        fileInfo = {

            path: file.name,
            encrypt: encrypt,
            content: fileContent,
        };

        // sending datas using fetch
        fetch("http://127.0.0.1:8080/api/encrypt", {

            // HTTP request - is using for sending datas
            method: "POST",
            // header - tells the server that we are sending JSON
            headers: { "Content-Type": "application/json", },
            // turns fileInfo into JSON
            // {
            //  path: "name.txt",
            //  encrypt: true/false,
            //  content: "some text content",
            // }
            body: JSON.stringify(fileInfo),
        })
            .then(response => response.text())
            .then(data => { 
            
                console.log("Responce: ", data)
                document.getElementById("file_content").value = data;
            })
            .catch(error => console.error("Error: ", error));
    }

    reader.readAsText(file);
}

document.getElementById('download_file').addEventListener('click', function() {
    
    fetch("/download")
        .then(response => {
            if (!response.ok) throw new Error("Download failed");

            // Получаем имя файла из заголовков
            let filename = "";
            let disposition = response.headers.get("Content-Disposition");
            if (disposition && disposition.includes("filename=")) {
                filename = disposition.split("filename=")[1].trim();
                filename = filename.replace(/["']/g, "");
            }

            return response.blob().then(blob => ({ blob, filename }));
        })
        .then(({ blob, filename }) => {
            let a = document.createElement("a");
            a.href = window.URL.createObjectURL(blob);
            // if there is no name set default
            a.download = filename || "downloaded_file"; 
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
        })
        .catch(error => console.error("Download error:", error));
});


document.getElementById("upload_file").addEventListener("click", function() {

    document.getElementById("file_input").click();
});

document.getElementById("file_input").addEventListener("change", function() {

    const file = this.files[0];
    if (!file)
        return;

    const formData = new FormData();
    formData.append("file", file);

    fetch("/upload", {

        method: "POST",
        body: formData,
    })
    .then(response => response.text())
    .then(data => console.log("FILE UPLOADED: ", data))
    .catch(error => console.log("FAILED TO UPLOAD FILE: ", error))

    const reader = new FileReader();
    reader.onload = function(event){

        document.getElementById("file_content").value = event.target.result;
    };

    reader.readAsText(file);
});