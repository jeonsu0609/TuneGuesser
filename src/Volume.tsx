import React, { useEffect, useRef, useState } from 'react'
// import { ProgressBar } from 'components/common'
import clsx from 'clsx'
import './Volume.scss'
import { volumeState } from './atoms'
import { useRecoilState } from 'recoil'

const VisualVolume = () => {
  const ref = useRef<HTMLDivElement>(null)
  const [mute, setMute] = useState(false)
  // const [volume, setVolume] = useState(50)

  const [volume, setVolume] = useRecoilState(volumeState)
  const [dragging, setDragging] = useState(false)

  useEffect(() => {
    const current = ref.current
    const handleMouseDown = () => {
      console.log('MouseDown', dragging)
      setDragging(true)
    }

    if (current) {
      current.addEventListener('mousedown', handleMouseDown)
    }

    return () => {
      if (current) {
        current.removeEventListener('mousedown', handleMouseDown)
      }
    }
  }, [])

  useEffect(() => {
    const dragMove = (e: any) => {
      console.log('dragMove', dragging, e)
      if (dragging) {
        onBarClick(e)
      }
    }

    const dragEnd = (e: any) => {
      console.log('dragEnd', dragging)
      setDragging(false)
      onBarClick(e)
    }

    if (dragging) {
      window.addEventListener('pointermove', dragMove)
      window.addEventListener('pointerup', dragEnd)
    }

    return () => {
      window.removeEventListener('pointermove', dragMove)
      window.removeEventListener('pointerup', dragEnd)
    }
  }, [dragging])

  const handleVolumeBtnClick = () => {
    setMute(!mute)
  }

  // const calculate = (e) => {
  //   const barRect = ref.current.getBoundingClientRect()
  //   const barStartX = barRect.left
  //   const barWidth = barRect.width

  //   let mouseX = e.pageX - barStartX
  //   mouseX = Math.max(mouseX, 0)
  //   mouseX = Math.min(mouseX, barWidth)

  //   const newPercent = (mouseX / barWidth) * 100
  //   let value

  //   value = !fixed
  //     ? Math.round((max / 100) * newPercent)
  //     : ((max / 100) * newPercent).toFixed(fixed)

  //   if (start) {
  //     value = Math.max(value, start)

  //     if (end) {
  //       value = Math.min(value, end)
  //     }
  //   }

  //   const roundedValue = Math.ceil(value * 2) / 2 // 0.5단위 반올림한 값

  //   return value
  // }

  const onBarClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!e.currentTarget.offsetParent) {
      return
    }

    const totalOffsetX =
      (e.currentTarget.offsetParent as HTMLDivElement).offsetLeft +
      e.currentTarget.offsetLeft

    const seekPoint = Math.round(
      ((e.clientX - totalOffsetX) * 100) / e.currentTarget.offsetWidth
    )

    setVolume(seekPoint)
  }
  const handleVolumeChangeStart = () => {}
  const handleVolumeChange = (value: any) => {}
  const handleVolumeChanged = (value: any) => {}

  return (
    <div ref={ref} className='root-volume'>
      <div className='bar' onClick={onBarClick}>
        <div className='progressbar'>
          <div className='progress' style={{ width: `${volume}%` }} />
        </div>
      </div>

      {mute ? (
        <button className='muteBtn' onClick={handleVolumeBtnClick}></button>
      ) : (
        <button className='volumeBtn' onClick={handleVolumeBtnClick}></button>
      )}
    </div>
  )
}

export default VisualVolume
