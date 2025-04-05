import React from 'react'
import { useRecoilValue } from 'recoil'
import { Song } from './entity'
import { songState } from './atoms'
import './Info.scss'

const Info = () => {
  const item = useRecoilValue<Song>(songState)
  return (
    <div className='root-info'>
      <div className='title'>{item.title}</div>
      <div className='artist'>{item.artist}</div>
    </div>
  )
}

export default Info
