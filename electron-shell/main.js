const { app, BrowserWindow } = require('electron')
const { spawn } = require('child_process')
const http = require('http')
const path = require('path')

let backendProcess
let mainWindow

function getBinaryPath() {
  const binaryName = process.platform === 'win32' ? 'desktop_email_client_backend.exe' : 'desktop_email_client_backend'
  if (app.isPackaged) {
    return path.join(process.resourcesPath, 'backend', binaryName)
  }
  return path.join(__dirname, '..', 'target', 'release', binaryName)
}

function getDistPath() {
  if (app.isPackaged) {
    return path.join(process.resourcesPath, "frontend", 'dist')
  }
  return path.join(__dirname, '..', "frontend", 'dist')
}

function startBackend() {
  const binaryPath = getBinaryPath()
  const distPath = getDistPath()

  backendProcess = spawn(binaryPath, [], {
    stdio: 'inherit',
    env: {
      ...process.env,
      DIST_PATH: distPath,
      PORT: '8080',
      RUST_LOG: 'info,desktop_email_client_backend=debug'
    }
  })

  backendProcess.on('error', (err) => {
    console.error('Failed to start backend:', err)
  })
}

function waitForBackend(retries = 20) {
  return new Promise((resolve, reject) => {
    const attempt = () => {
      http.get('http://localhost:8080', () => resolve()).on('error', () => {
        if (retries-- > 0) setTimeout(attempt, 200)
        else reject(new Error('Backend never started'))
      })
    }
    attempt()
  })
}

function createWindow() {
  mainWindow = new BrowserWindow({
    show: false,
    width: 1280,
    height: 800,
    webPreferences: {
      contextIsolation: true
    },
    autoHideMenuBar: true
  })

  mainWindow.loadURL('http://localhost:8080')

  mainWindow.once('ready-to-show', () => {
    mainWindow.show()
  })
}

app.whenReady().then(async () => {
  startBackend()
  await waitForBackend()
  createWindow()
})

app.on('window-all-closed', () => {
  if (backendProcess) backendProcess.kill()
  app.quit()
})
