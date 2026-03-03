const { app, BrowserWindow } = require('electron')
const { spawn } = require('child_process')
const http = require('http')
const path = require('path')
const fs = require('fs')
const os = require('os')

let backendProcess
let mainWindow

function getBinaryPath() {
  const binaryName = process.platform === 'win32' ? 'desktop_email_client_backend.exe' : 'desktop_email_client_backend'
  if (app.isPackaged) {
    return path.join(process.resourcesPath, 'backend', binaryName)
  }
  return path.join(__dirname, '..', 'target', 'release', binaryName)
}

function getFrontendDistPath() {
  if (app.isPackaged) {
    return path.join(process.resourcesPath, "frontend", 'dist')
  }
  return path.join(__dirname, '..', "frontend", 'dist')
}

function getDatabasePath() {
  return path.join(app.getPath('userData'), 'db.sqlite')
}

function getBackendPortPath() {
	return path.join(os.tmpdir(), 'desktop_email_client_backend.port')

}

function startBackend() {
  const binaryPath = getBinaryPath()
  const frontendDistPath = getFrontendDistPath()

  backendProcess = spawn(binaryPath, [], {
    stdio: 'inherit',
    env: {
      ...process.env,
      FRONTEND_DIST_PATH: frontendDistPath,
      DATABASE_URL: 'sqlite://' + getDatabasePath(),
      RUST_LOG: 'info,desktop_email_client_backend=debug'
    }
  })

  backendProcess.on('error', (err) => {
    console.error('Failed to start backend:', err)
  })
}

function waitForBackendPort(portFile, retries = 25) {
  return new Promise((resolve, reject) => {
    const attempt = () => {
      try {
        const port = parseInt(fs.readFileSync(portFile, 'utf8').trim())
        if (!isNaN(port)) {
          fs.unlinkSync(portFile)
          resolve(port)
        } else {
          retry()
        }
      } catch {
        retry()
      }
    }
    const retry = () => {
      if (retries-- > 0) setTimeout(attempt, 200)
      else reject(new Error('Backend never wrote port file'))
    }
    attempt()
  })
}

function waitForBackend(port, retries = 20) {
  return new Promise((resolve, reject) => {
    const attempt = () => {
      http.get('http://localhost:' + port + '/api/health', () => resolve()).on('error', () => {
        if (retries-- > 0) setTimeout(attempt, 200)
        else reject(new Error('Backend never started'))
      })
    }
    attempt()
  })
}

function createWindow(port) {
  mainWindow = new BrowserWindow({
    show: false,
    width: 1280,
    height: 800,
    webPreferences: {
      contextIsolation: true
    },
    autoHideMenuBar: true
  })

  mainWindow.loadURL('http://localhost:' + port)

  mainWindow.once('ready-to-show', () => {
    mainWindow.show()
  })
}

app.whenReady().then(async () => {
  portFilepath = getBackendPortPath()
  try { fs.rmSync(portFilepath) } catch (_e) {}
  startBackend()
  const port = await waitForBackendPort(portFilepath)
  await waitForBackend(port)
  createWindow(port)
})

app.on('window-all-closed', () => {
  if (backendProcess) backendProcess.kill()
  app.quit()
})
