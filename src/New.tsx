import React, { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import './New.scss'

const New: React.FC = () => {
  const [isPlaying, setIsPlaying] = useState(false)
  const [currentTime, setCurrentTime] = useState(0)
  const [duration, setDuration] = useState(0)
  const audioRef = useRef<HTMLAudioElement>(null)

  useEffect(() => {
    const unlisten = listen('event-name', (event) => {
      console.log(event)
    })

    const song = listen('song', (event) => {
      console.log(event)
    })

    const state = listen('state', (event) => {
      console.log(event)
    })

    const login = listen('login', (event) => {
      console.log(event)
    })

    return () => {
      unlisten.then((f) => f())
      song.then((f) => f())
      state.then((f) => f())
      login.then((f) => f())
    }
  }, [])

  const togglePlay = () => {
    if (isPlaying) {
      audioRef.current?.pause()
    } else {
      audioRef.current?.play()
    }
    setIsPlaying(!isPlaying)
  }

  const handleTimeUpdate = () => {
    if (audioRef.current) {
      setCurrentTime(audioRef.current.currentTime)
      setDuration(audioRef.current.duration)
    }
  }

  const handleSeek = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (audioRef.current) {
      audioRef.current.currentTime = Number(event.target.value)
      setCurrentTime(audioRef.current.currentTime)
    }
  }

  return (
    <div className='player'>
      <div className='player__album'>
        <img src='album-art.jpg' alt='Album Art' />
      </div>
      <div className='player__controls'>
        <button onClick={() => invoke('previous_song', {})}>⏮</button>
        <button onClick={togglePlay}>{isPlaying ? '⏸' : '▶️'}</button>
        <button onClick={() => invoke('next_song', {})}>⏭</button>
      </div>
      <div className='player__progress'>
        <input
          type='range'
          min='0'
          max={duration}
          value={currentTime}
          onChange={handleSeek}
        />
        <div className='player__time'>
          <span>
            {Math.floor(currentTime / 60)}:
            {Math.floor(currentTime % 60)
              .toString()
              .padStart(2, '0')}
          </span>
          <span>
            {Math.floor(duration / 60)}:
            {Math.floor(duration % 60)
              .toString()
              .padStart(2, '0')}
          </span>
        </div>
      </div>
      <div className='player__volume'>
        <input
          type='range'
          min='0'
          max='100'
          onChange={(e) => {
            if (audioRef.current) {
              audioRef.current.volume = Number(e.target.value) / 100
            }
          }}
        />
      </div>
      <audio ref={audioRef} onTimeUpdate={handleTimeUpdate} />
    </div>
  )
}

export default New
