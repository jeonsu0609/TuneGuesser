import React, { useEffect } from 'react'
import './Controller.scss'
import Volume from './Volume'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { useRecoilState } from 'recoil'
import { isPlayingState } from './atoms'

const VisualController = () => {
  const [playing, setPlaying] = useRecoilState(isPlayingState)

  const hadlePlayOrPauseBtnClick = () => {
    console.log('hadlePlayOrPauseBtnClick')
    setPlaying(!playing)
  }
  const handlePrevBtnClick = () => {
    console.log('handlePrevBtnClick')
    invoke('prev', {})
  }
  const handleNextBtnClick = () => {
    console.log('handleNextBtnClick')
    invoke('next', {})
  }

  return (
    <div className='root-control'>
      <div className='controller'>
        <button className='prev' onClick={handlePrevBtnClick}></button>
        {playing && (
          <button className='pause' onClick={hadlePlayOrPauseBtnClick}></button>
        )}
        {!playing && (
          <button className='play' onClick={hadlePlayOrPauseBtnClick}></button>
        )}
        <button className='next' onClick={handleNextBtnClick}></button>
        <Volume />
      </div>
    </div>
  )
}

export default VisualController
