import React from 'react'
import { createRoot } from 'react-dom/client'

import './index.scss'
import App from './App'
import Audio from './Audio'
import { RecoilRoot } from 'recoil'
import { Route } from 'react-router-dom'

const container = document.getElementById('app')
const root = createRoot(container!)
root.render(
  <RecoilRoot>
    {/* <Route path={'/'} element={<App />}></Route> */}
    <App />
    <Audio />
  </RecoilRoot>
)
