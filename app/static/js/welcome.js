window.tokenRevealTimeout = null; // Make global
let originalToken = null;

function showTokenActions() {
    document.getElementById('token-actions').style.display = 'flex';
}

function hideTokenActions() {
    document.getElementById('token-actions').style.display = 'none';
}

function checkTokenChanged() {
    const input = document.getElementById('token-input');
    if (originalToken !== null && input.value !== originalToken) {
        hideTokenActions();
        originalToken = null;
    }
}

document.getElementById('token-input').addEventListener('input', checkTokenChanged);

document.getElementById('generate-token').addEventListener('click', async () => {
    const btn = document.getElementById('generate-token');
    const input = document.getElementById('token-input');

    // Disable button while fetching
    btn.disabled = true;
    btn.textContent = 'Generating...';

    try {
        const response = await fetch('/api/v1/auth/token', {
            method: 'GET',
            headers: { 'Content-Type': 'application/json' }
        });

        if (!response.ok) {
            throw new Error('Failed to generate token');
        }

        const data = await response.json();
        input.value = data.token;

        // Clear any existing timeout to avoid multiple rapid clicks causing issues
        if (tokenRevealTimeout) {
            clearTimeout(tokenRevealTimeout);
        }

        // Temporarily show as plain text
        input.type = 'text';

        // Countdown timer (synced with tokenRevealTimeout)
        const cooldownMs = 5000;
        let countdown = Math.ceil(cooldownMs / 1000);
        btn.textContent = `Your token is ready! Save it securely. (${countdown}s)`;

        if (window.tokenRevealTimeout) {
            clearTimeout(window.tokenRevealTimeout);
            window.tokenRevealTimeout = null; // Clear global reference
        }

        window.countdownInterval = setInterval(() => { // Make global
            countdown--;
            if (countdown > 0) {
                btn.textContent = `Your token is ready! Save it securely. (${countdown}s)`;
            } else {
                clearInterval(window.countdownInterval);
                window.countdownInterval = null; // Clear global reference
                input.type = 'password';
                btn.textContent = 'Generate New Token';
                btn.disabled = false;
            }
        }, 1000);

        // Store original token and show action buttons
        originalToken = data.token;
        showTokenActions();
    } catch (error) {
        alert('Error generating token: ' + error.message);
        btn.disabled = false;
        btn.textContent = 'Generate New Token';
    }
});

document.getElementById('auth-btn').addEventListener('click', async () => {
    const token = document.getElementById('token-input').value;
    const input = document.getElementById('token-input');
    const generateBtn = document.getElementById('generate-token'); // Get generate-token button

    // Clear any active countdown on generate-token button
    if (window.tokenRevealTimeout) {
        clearTimeout(window.tokenRevealTimeout);
        window.tokenRevealTimeout = null;
    }
    if (window.countdownInterval) {
        clearInterval(window.countdownInterval);
        window.countdownInterval = null;
    }
    if (generateBtn) {
        generateBtn.disabled = false;
        generateBtn.textContent = 'Generate New Token';
    }

    // Ensure token field is password type
    input.type = 'password';

    if (!token) {
        alert('Please enter or generate a token.');
        return;
    }

    try {
        const response = await fetch('/api/v1/auth/login_or_register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ token: token })
        });

        if (response.ok) {
            window.location.href = '/dashboard';
        } else {
            throw new Error('Authentication failed. Please check your token.');
        }
    } catch (error) {
        alert(error.message);
    }
});

document.getElementById('copy-btn').addEventListener('click', async () => {
    const token = document.getElementById('token-input').value;
    const input = document.getElementById('token-input');
    const copyBtn = document.getElementById('copy-btn');
    const originalText = copyBtn.textContent;

    try {
        // Try modern Clipboard API first
        if (navigator.clipboard && navigator.clipboard.writeText) {
            await navigator.clipboard.writeText(token);
        } else {
            // Fallback for Safari and older browsers
            input.type = 'text';
            input.select();
            document.execCommand('copy');
            input.type = 'password';
        }
        copyBtn.textContent = 'Copied!';
        setTimeout(() => {
            copyBtn.textContent = originalText;
        }, 1500);
    } catch (error) {
        alert('Failed to copy to clipboard: ' + error.message);
    }
});

document.getElementById('save-btn').addEventListener('click', () => {
    const token = document.getElementById('token-input').value;
    const blob = new Blob([token], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'cryptowrap-bearer-token.txt';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
});

function generateToken() {
    let token = '';
    return token;
}
