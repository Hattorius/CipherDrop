const button = document.querySelector('button');
const loader = document.querySelector('.loader');
const availability = document.querySelector('.availability');
const errorP = document.querySelector('.error');

const uuid = document.querySelector('input#uuid').value;
const availableTill = parseInt(document.querySelector('input#available_till').value);
var iv = document.querySelector('input#iv').value;
var key = document.querySelector('input#key').value;
const mimeType = document.querySelector('input#mime_type').value;
const fileName = document.querySelector('input#file_name').value;

var file = null;
var state = 0;
var currentProgress = 0;
var intervalId;


const startProgress = () => {
    intervalId = setInterval(() => {
        if (currentProgress < 95 && state === 1) {
            currentProgress += 2.5;
        } else if (state === 2) {
            currentProgress = 100;
            clearInterval(intervalId);
        }
        progress(currentProgress);
    }, 350);
}

const stopProgress = () => {
    clearInterval(intervalId);
    setTimeout(() => {
        state = 0;
        currentProgress = 0;
    }, 350);
    loader.classList.add('hidden');
}

button.addEventListener('click', () => {
    if (file !== null) {
        return handleFile(null);
    }

    startProgress();

    const xhr = new XMLHttpRequest();
    xhr.open('GET', `/api/file/${uuid}/download`, true);
    xhr.responseType = 'arraybuffer';

    xhr.onprogress = event => {
        if (event.lengthComputable) {
            const percentComplete = (event.loaded / event.total) * 80;
            currentProgress = percentComplete;
        }
    };

    xhr.onload = () => {
        if (xhr.status === 200) {
            handleFile(xhr.response).catch(() => error("Failed decrypting file."));
        } else {
            const data = JSON.parse(xhr.responseText);
            error(data.message);
        }
    }

    xhr.onerror = () => {
        error('An error occured while downloading file.');
    }

    xhr.send();
});

const progress = percent => {
    loader.classList.remove('hidden');
    loader.querySelector('div.inner').style.width = `${percent}%`;
}

const success = () => {
    state = 2;
}

const error = msg => {
    stopProgress();
    errorP.classList.remove('hidden');
    errorP.innerText = msg;
}


const handleFile = async bytes => {
    state = 1;
    currentProgress = 80;

    if (file === null) {
        if (iv === "" || key === "") {
            const fragments = window.location.hash.substring(1).split('~');
            iv = fragments[0];
            key = fragments[1];
        }

        file = await decryptFile(
            bytes,
            key,
            iv,
            mimeType,
            fileName
        );
    }

    const url = URL.createObjectURL(file);

    const a = document.createElement('a');
    a.href = url;
    a.download = file.name;
    a.style.display = 'none';
    document.body.appendChild(a);

    a.onclick = () => {
        setTimeout(() => {
            URL.revokeObjectURL(url)
            document.body.removeChild(a);
        }, 500);
    };

    success();

    setTimeout(() => {
        a.click();
    }, 350);
}

const base64UrlToArrayBuffer = string => {
    const base64String = string.replace(/-/g, '+').replace(/_/g, '/');
    const binaryString = atob(base64String);
    const len = binaryString.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
}

const decryptFile = async (encryptedData, encodedKey, encodedIv, mimeType, fileName) => {
    const rawKey = base64UrlToArrayBuffer(encodedKey);
    const iv = base64UrlToArrayBuffer(encodedIv);

    const key = await crypto.subtle.importKey(
        'raw',
        rawKey,
        {
            name: 'AES-GCM',
            length: 256
        },
        false,
        [
            "decrypt"
        ]
    );

    const decryptedData = await crypto.subtle.decrypt(
        {
            name: 'AES-GCM',
            iv
        },
        key,
        encryptedData
    );

    return new File(
        [
            decryptedData
        ],
        fileName,
        {
            type: mimeType
        }
    );
}


// Calculate time remaining and put it in the HTML
const now = Math.floor(Date.now() / 1000);
const timeLeft = availableTill - now;

if (timeLeft <= 0) {
    availability.innerText = "This file is unavailable";
} else {
    const days = Math.floor(timeLeft / (60 * 60 * 24));
    const hours = Math.floor(timeLeft / (60 * 60));
    const minutes = Math.floor(timeLeft / 60);

    if (days > 0) {
        availability.innerText = `File available for the next ${days} day${days > 1 ? 's' : ''}`
    } else if (hours > 0) {
        availability.innerText = `File available for the next ${hours} hour${hours > 1 ? 's' : ''}`
    } else if (minutes > 0) {
        availability.innerText = `File available for the next ${minutes} minute${minutes > 1 ? 's' : ''}`
    } else {
        availability.innerText = "File is available for the next <1 minute"
    }
}
