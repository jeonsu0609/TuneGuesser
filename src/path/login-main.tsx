import React from 'react'
import ReactDOM from 'react-dom/client'
import Window_View from './login'

ReactDOM.createRoot(
  document.getElementById('login-root') as HTMLElement
).render(
  // <React.StrictMode>
  <Window_View />
  // </React.StrictMode>
)
