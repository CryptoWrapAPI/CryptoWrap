// Landing Page - Welcome Modal

// Check if user is already authenticated via cookie
function checkAuthCookie() {
    const cookies = document.cookie.split(';').map(c => c.trim());
    const hasAuth = cookies.some(c => c.startsWith('auth_js'));
    if (hasAuth) {
        window.location.href = '/dashboard';
    }
}

document.addEventListener('DOMContentLoaded', () => {
    // checkAuthCookie();

    const openModalBtn = document.getElementById('open-auth-modal');
    if (!openModalBtn) return;

    openModalBtn.addEventListener('click', openWelcomeModal);
});

function openWelcomeModal() {

    // checkAuthCookie();
    const cookies = document.cookie.split(';').map(c => c.trim());
    const hasAuth = cookies.some(c => c.startsWith('auth_js'));
    if (hasAuth) {
        window.location.href = '/dashboard';
        return;
    }

    
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.id = 'welcome-modal';
    // <p class="welcome-subtitle">Login or Register with your token</p>
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Login or Register with your token</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">&times;</button>
            </div>
            <div class="modal-body">
                <div class="welcome-form">
                    <div class="input-group">
                        <input type="password" id="token-input-modal" placeholder="Enter your token">
                        <div class="token-actions" id="token-actions-modal" style="display: none;">
                            <button id="copy-btn-modal" class="btn-copy">Copy</button>
                            <button id="save-btn-modal" class="btn-save">Save to File</button>
                        </div>
                    </div>
                    <button id="auth-btn-modal" class="btn-primary">Authenticate</button>
                    <button id="generate-token-modal" class="btn-secondary">Generate New Token</button>
                </div>
            </div>
        </div>
    `;

    document.body.appendChild(modal);
    addModalStyles();

    // Event listeners
    const closeBtn = modal.querySelector('.modal-close');
    const generateBtn = modal.querySelector('#generate-token-modal');
    const authBtn = modal.querySelector('#auth-btn-modal');
    const tokenInput = modal.querySelector('#token-input-modal');
    const copyBtn = modal.querySelector('#copy-btn-modal');
    const saveBtn = modal.querySelector('#save-btn-modal');
    const tokenActions = modal.querySelector('#token-actions-modal');

    closeBtn.addEventListener('click', () => modal.remove());
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });

    // Generate token
    generateBtn.addEventListener('click', async () => {
        const btn = generateBtn;
        const input = tokenInput;

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

            if (window.tokenRevealTimeout) {
                clearTimeout(window.tokenRevealTimeout);
            }

            input.type = 'text';

            const cooldownMs = 5000;
            let countdown = Math.ceil(cooldownMs / 1000);
            btn.textContent = `Your token is ready! Save it securely. (${countdown}s)`;

            if (window.tokenRevealTimeout) {
                clearTimeout(window.tokenRevealTimeout);
                window.tokenRevealTimeout = null;
            }

            window.countdownInterval = setInterval(() => {
                countdown--;
                if (countdown > 0) {
                    btn.textContent = `Your token is ready! Save it securely. (${countdown}s)`;
                } else {
                    clearInterval(window.countdownInterval);
                    window.countdownInterval = null;
                    input.type = 'password';
                    btn.textContent = 'Generate New Token';
                    btn.disabled = false;
                }
            }, 1000);

            window.originalToken = data.token;
            tokenActions.style.display = 'flex';
        } catch (error) {
            alert('Error generating token: ' + error.message);
            btn.disabled = false;
            btn.textContent = 'Generate New Token';
        }
    });

    // Check if token changed
    tokenInput.addEventListener('input', () => {
        if (window.originalToken !== null && tokenInput.value !== window.originalToken) {
            tokenActions.style.display = 'none';
            window.originalToken = null;
        }
    });

    // Copy button
    copyBtn.addEventListener('click', async () => {
        const token = tokenInput.value;
        const copyBtn = document.getElementById('copy-btn-modal');
        const originalText = copyBtn.textContent;

        try {
            if (navigator.clipboard && navigator.clipboard.writeText) {
                await navigator.clipboard.writeText(token);
            } else {
                tokenInput.type = 'text';
                tokenInput.select();
                document.execCommand('copy');
                tokenInput.type = 'password';
            }
            copyBtn.textContent = 'Copied!';
            setTimeout(() => {
                copyBtn.textContent = originalText;
            }, 1500);
        } catch (error) {
            alert('Failed to copy to clipboard: ' + error.message);
        }
    });

    // Save button
    saveBtn.addEventListener('click', () => {
        const token = tokenInput.value;
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

    // Authenticate
    authBtn.addEventListener('click', async () => {
        const token = tokenInput.value;
        const input = tokenInput;
        const generateBtn = document.getElementById('generate-token-modal');

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
                modal.remove();
                window.location.href = '/dashboard';
            } else {
                throw new Error('Authentication failed. Please check your token.');
            }
        } catch (error) {
            alert(error.message);
        }
    });
}

function addModalStyles() {
    if (document.getElementById('modal-styles')) return;

    const style = document.createElement('style');
    style.id = 'modal-styles';
    style.textContent = `
        .modal-overlay {
            display: flex;
            align-items: center;
            justify-content: center;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.7);
            z-index: 1000;
            backdrop-filter: blur(4px);
        }

        .modal-content {
            background: #020204d6;
            border: 1px solid rgba(100, 150, 255, 0.2);
            border-radius: 12px;
            max-width: 500px;
            width: 90%;
            max-height: 90vh;
            overflow-y: auto;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.7);
        }

        .modal-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1.5rem;
            border-bottom: 1px solid rgba(100, 150, 255, 0.2);
        }

        .modal-header h3 {
            color: #6a96ff;
            font-size: 1.25rem;
            margin: 0;
        }

        .modal-close {
            background: none;
            border: none;
            color: #888;
            font-size: 1.5rem;
            cursor: pointer;
            padding: 0;
            width: 30px;
            height: 30px;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: color 0.3s ease;
        }

        .modal-close:hover {
            color: #e0e0e0;
        }

        .modal-body {
            padding: 1.5rem;
        }

        .welcome-form {
            display: flex;
            flex-direction: column;
            <!-- gap: 1rem; -->
        }

        .welcome-subtitle {
            text-align: center;
            color: #888;
            margin-bottom: 0.5rem;
            font-size: 0.9rem;
        }

        .input-group {
            margin-bottom: 0.5rem;
        }

        input[type="text"],
        input[type="password"] {
            width: 100%;
            padding: 0.75rem;
            border: 2px solid #333;
            border-radius: 8px;
            font-size: 1rem;
            background: #0f0f1a;
            color: #e0e0e0;
            box-sizing: border-box;
        }

        input[type="text"]:focus,
        input[type="password"]:focus {
            outline: none;
            border-color: #6496ff;
        }

        input[type="text"]::placeholder,
        input[type="password"]::placeholder {
            color: #555;
        }

        .token-actions {
            display: flex;
            gap: 0.5rem;
            margin-top: 0.5rem;
        }

        .token-actions button {
            flex: 1;
            padding: 0.5rem;
            font-size: 0.875rem;
        }

        .btn-copy {
            background: transparent;
            color: #6a3ad9;
            border: 2px solid #6a3ad9;
        }

        .btn-copy:hover {
            background: #6a3ad9;
            color: white;
        }

        .btn-save {
            background: transparent;
            color: #3a5fc8;
            border: 2px solid #3a5fc8;
        }

        .btn-save:hover {
            background: #3a5fc8;
            color: white;
        }

        .btn-primary,
        .btn-secondary {
            width: 100%;
            padding: 0.75rem;
            border: none;
            border-radius: 8px;
            font-size: 1rem;
            cursor: pointer;
            box-sizing: border-box;
        }

        .btn-primary {
            background: linear-gradient(135deg, #3a5fc8 0%, #6a3ad9 100%);
            color: white;
        }

        .btn-primary:hover {
            background: linear-gradient(135deg, #6a3ad9 0%, #3a5fc8 100%);
        }

        .btn-secondary {
            background: transparent;
            color: #3a5fc8;
            border: 2px solid #3a5fc8;
        }

        .btn-secondary:hover {
            background: #3a5fc8;
            color: white;
        }

        .btn-primary:active,
        .btn-secondary:active {
            transform: scale(0.98);
        }

        .btn-primary:disabled,
        .btn-secondary:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }

        @media (max-width: 480px) {
            .modal-content {
                width: 95%;
            }

            .modal-header {
                padding: 1rem;
            }

            .modal-body {
                padding: 1rem;
            }
        }
    `;
    document.head.appendChild(style);
}
