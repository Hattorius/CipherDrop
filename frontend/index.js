const dropZone = document.querySelector('div.drop');
const button = document.querySelector('button');
const fileInput = document.querySelector('input');
const uploads = document.querySelector('.uploads');
const select = document.querySelector('select');

const uploadingTemplate = document.querySelector('.uploading.hidden').cloneNode(true);
const successTemplate = document.querySelector('.success.hidden').cloneNode(true);
const failTemplate = document.querySelector('.fail.hidden').cloneNode(true);

uploadingTemplate.classList.remove('hidden');
successTemplate.classList.remove('hidden');
failTemplate.classList.remove('hidden');

dropZone.addEventListener('dragover', e => e.preventDefault());
dropZone.addEventListener('drop', e => {
    e.preventDefault();
    const files = Array.from(e.dataTransfer.files);
    handleFiles(files);
});
button.addEventListener('click', () => fileInput.click());
fileInput.addEventListener('change', () => {
    const files = Array.from(fileInput.files);
    handleFiles(files);
});


class FileUpload {
    constructor(file) {
        this.file = file;
        this.fileType = file.type;
        this.fileName = file.name;
        this.templateHolder = document.createElement('div');
        uploads.appendChild(this.templateHolder);
    }

    async start(lifetime) {
        if (this.file.size > 1073741824) {
            return this.error('File size exceeds 1GB limit.')
        }

        this.progress();
        const file = await encryptFile(this.file);
        if (file.data.length > 1073741824) {
            return this.error('File size exceeds 1GB limit.')
        }

        const fileBlob = new Blob([file.data], { type: 'application/octet-stream' });

        const xhr = new XMLHttpRequest();
        const formData = new FormData();

        xhr.open('POST', '/api/upload', true);
        formData.append('file_name', this.fileName);
        formData.append('file_type', file.mimeType);
        formData.append('lifetime', lifetime);
        formData.append('file', fileBlob, 'file');

        xhr.upload.onprogress = event => {
            if (event.lengthComputable) {
                let percentComplete = (event.loaded / event.total) * 100;
                this.progress(percentComplete);
            }
        };

        xhr.onload = () => {
            const body = JSON.parse(xhr.responseText);

            if (body.success) {
                this.success(`${window.location.protocol}//${window.location.hostname}/file/${body.uuid}#${file.iv}~${file.key}`);
            } else {
                this.error(body.message);
            }
        }

        xhr.onerror = () => {
            this.error('An error occured while uploading file.');
        }

        xhr.send(formData);
    }

    progress(percent = 0) {
        const progress = uploadingTemplate.cloneNode(true);
        progress.querySelector('span').innerText = this.fileName;
        progress.querySelector('div.inner').style.width = `${percent}%`;
        this.templateHolder.innerHTML = '';
        this.templateHolder.appendChild(progress);
    }

    success(link) {
        const success = successTemplate.cloneNode(true);
        const input = success.querySelector('input');
        success.querySelector('span').innerText = this.fileName;
        success.querySelector('button').addEventListener('click', () => {
            input.select();
            input.setSelectionRange(0, 99999);
            navigator.clipboard.writeText(input.value);
        });
        input.value = link;
        this.templateHolder.innerHTML = '';
        this.templateHolder.appendChild(success);
    }

    error(msg) {
        const error = failTemplate.cloneNode(true);
        error.querySelector('span').innerText = this.fileName;
        error.querySelector('div').innerText = msg;
        this.templateHolder.innerHTML = '';
        this.templateHolder.appendChild(error);
    }
}

const handleFile = async file => {
    const fileUploader = new FileUpload(file);

    fileUploader.start(select.value);
}

const handleFiles = files => { // FileList
    for (const file of files) {
        handleFile(file);
    }
}

const fileToArrayBuffer = file => {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result);
        reader.onerror = error => reject(error);
        reader.readAsArrayBuffer(file);
    });
}

const arrayBufferToBase64Url = buffer => {
    const base64String = btoa(String.fromCharCode(...buffer));
    return base64String.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

const generateKey = async () => {
    return crypto.subtle.generateKey(
        {
            name: "AES-GCM",
            length: 256
        },
        true,
        [
            "encrypt",
            "decrypt"
        ]
    );
}

const encryptFile = async file => {
    const key = await generateKey();
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const fileBuffer = await fileToArrayBuffer(file);

    const encryptedData = await crypto.subtle.encrypt(
        {
            name: "AES-GCM",
            iv
        },
        key,
        fileBuffer
    );

    const exportedKey = await crypto.subtle.exportKey('raw', key);
    return {
        data: new Uint8Array(encryptedData),
        key: arrayBufferToBase64Url(new Uint8Array(exportedKey)),
        iv: arrayBufferToBase64Url(iv),
        mimeType: file.type,
        fileType: file.name.includes('.') ? file.name.split('.').pop() : null // .png?
    }
}
