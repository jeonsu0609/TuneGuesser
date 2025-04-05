import React, { useState, useRef } from 'react'
import './Footer.scss'

const Footer: React.FC = () => {
  const [isPlaying, setIsPlaying] = useState(false)
  const [currentTime, setCurrentTime] = useState(0)
  const [duration, setDuration] = useState(0)
  const audioRef = useRef<HTMLAudioElement>(null)

  const togglePlayPause = () => {
    if (audioRef.current) {
      if (isPlaying) {
        audioRef.current.pause()
      } else {
        audioRef.current.play()
      }
      setIsPlaying(!isPlaying)
    }
  }

  const handleTimeUpdate = () => {
    if (audioRef.current) {
      setCurrentTime(audioRef.current.currentTime)
    }
  }

  const handleLoadedData = () => {
    if (audioRef.current) {
      setDuration(audioRef.current.duration)
    }
  }

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (audioRef.current) {
      const seekTime = parseFloat(e.target.value)
      audioRef.current.currentTime = seekTime
      setCurrentTime(seekTime)
    }
  }

  const formatTime = (time: number) => {
    const minutes = Math.floor(time / 60)
    const seconds = Math.floor(time % 60)
    return `${minutes}:${seconds < 10 ? '0' : ''}${seconds}`
  }

  return (
    <div className='footer'>
      <div className='playback-controls'>
        <button onClick={togglePlayPause}>
          {isPlaying ? 'Pause' : 'Play'}
        </button>
      </div>
      <div className='seek-bar'>
        <span>{formatTime(currentTime)}</span>
        <input
          type='range'
          value={currentTime}
          max={duration}
          onChange={handleSeek}
        />
        <span>{formatTime(duration)}</span>
      </div>
      <div className='volume-control'>
        <input type='range' min='0' max='1' step='0.01' />
      </div>
      <audio
        ref={audioRef}
        src='https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3'
        onTimeUpdate={handleTimeUpdate}
        onLoadedData={handleLoadedData}
      />
    </div>
  )
}

export default Footer
