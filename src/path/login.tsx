import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import React from 'react'
import { WebviewWindow } from '@tauri-apps/api/window'
import { http } from '@tauri-apps/api'

function Window_View() {
  const [url, setUrl] = useState('')

  const handleLogin = (info: any) => {
    console.log('login', info)
    invoke('simple_command_with_result', {
      argument: '2024'.toString(),
    }).then((e) => console.log(e))
  }

  const handleOpenPopup = async () => {
    document.location.assign('http://localhost:3000/')

    document.addEventListener('login', handleLogin)
    window.addEventListener('login', handleLogin)
  }

  const observeUrlChange = () => {
    let oldHref = document.location.href
    const body = document.querySelector('body')
    console.log(body)
    const observer = new MutationObserver((mutations) => {
      if (oldHref !== document.location.href) {
        oldHref = document.location.href
        console.log('abc')
        handleLogin(true)
      }
    })
    if (body) observer.observe(body, { childList: true, subtree: true })
  }

  window.onload = observeUrlChange

  useEffect(() => {
    handleOpenPopup()

    return () => {
      document.removeEventListener('login', handleLogin)
      window.removeEventListener('login', handleLogin)
    }
  }, [])

  return (
    <div className='container'>
      <h1>Window</h1>
      <button onClick={handleOpenPopup}>Hello</button>
      <button onClick={handleLogin}>Tauri</button>
    </div>
  )
}

export default Window_View
