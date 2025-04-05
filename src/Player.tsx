import { useEffect, useState } from 'react'
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil'
import { emit, listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri'
import img from '../assets/img/logo.png'
import { isPlayingState, progressState, songState } from './atoms'
import { Song } from './entity'
import Info from './Info'
import './Player.scss'
import Visual from './Visual'
import Controller from './Controller'
import Progressbar from './Progressbar'

function arrayBufferToBase64(buffer: number[]) {
  let binary = ''
  const bytes = [].slice.call(new Uint8Array(buffer))
  bytes.forEach((b) => {
    binary += String.fromCharCode(b)
  })
  return window.btoa(binary)
}

// await listen('rs2js', (e) => {
//   console.log(e)
// })

const Player = () => {
  const [item, setItem] = useRecoilState<Song>(songState)
  const [isPlaying, setIsPlaying] = useRecoilState(isPlayingState)
  const setProgressBar = useSetRecoilState(progressState)
  const [image, setImage] = useState('')

  //   function updateWidth() {
  //     setProgressBar((sound.seek() / sound.duration()) * 100)
  //   }

  //   setInterval(() => {
  //     updateWidth()
  //   }, 100)

  //   window.electron.ipcRenderer.once('getAssetsPath', (arg) => {
  //     const songAssetsPath = (arg as string).replaceAll('\\', '/')
  //     setSongPath(songAssetsPath)
  //   })

  //   window.electron.ipcRenderer.once('getSongTitle', (arg) => {
  //     setSongTitle(arg as string)
  //   })

  //   window.electron.ipcRenderer.once('getAlbumImage', (arg) => {
  //     const base64Flag = 'data:image/jpeg;base64,'
  //     const imageStr = arrayBufferToBase64((arg as AlbumImageModel).data.data)
  //     setImage(base64Flag + imageStr)
  //   })

  //   window.electron.ipcRenderer.once('getSongFile', (arg) => {
  //     setSongFile(arg as string)
  //   })

  //   window.electron.ipcRenderer.once('getSongId', (arg) => {
  //     setSongId(arg as string)
  //   })

  return (
    <div className='root-player'>
      <div className='container'>
        <div className='top'>
          <Info />
        </div>

        <div className='middle'>
          <Visual />
        </div>

        <div className='bottom'>
          <div className='progressbar-container'>
            <Progressbar />
          </div>
          <div>
            <Controller />
          </div>
        </div>
      </div>
    </div>
  )
}

export default Player
