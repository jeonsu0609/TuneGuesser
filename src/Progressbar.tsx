import React, { useEffect, useRef, useState } from 'react'
import { useRecoilState, useRecoilValue } from 'recoil'
import { progressState, songState, seekState } from './atoms'
import { Song } from './entity'
import './Progressbar.scss'

const Progressbar = () => {
  const [current, setCurrent] = useRecoilState(progressState)
  const [seek, setSeek] = useRecoilState(seekState)
  const [currentTime, setCurrentTime] = useState(0)
  const item = useRecoilValue<Song>(songState)
  const [duration, setDuration] = useState(Number(item.duration))
  const progressRef = useRef<HTMLDivElement>(null)

  const handleSeek = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
    if (!progressRef.current) return

    const rect = progressRef.current.getBoundingClientRect()
    const offsetX = event.clientX - rect.left
    const newTime = (offsetX / rect.width) * duration
    setSeek(newTime)
  }

  useEffect(() => {
    setCurrentTime(current)
  }, [current])

  useEffect(() => {
    setDuration(Number(item.duration))
  }, [item.duration])

  return (
    <div className='progressbar' ref={progressRef} onClick={handleSeek}>
      <div
        className='progressbar__filled'
        style={{ width: `${(currentTime / duration) * 100}%` }}
      ></div>
    </div>
  )
}

export default Progressbar
