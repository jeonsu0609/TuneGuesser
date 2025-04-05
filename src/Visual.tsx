import React from 'react'
import clsx from 'clsx'
import './Visual.scss'
import { useRecoilValue } from 'recoil'
import { Song } from './entity'
import { songState } from './atoms'

const Visual = () => {
  const item = useRecoilValue<Song>(songState)

  return (
    <div className='root-visual'>
      {/* <div className='topSpace'></div> */}
      <div className='container'>
        <div className='albumImage'>
          {/* <div className='left'></div> */}
          <div className='content'>
            <img className='album-img' src={item.img} alt='AlbumImage' />
          </div>
          {/* <div className='right'></div> */}
        </div>
      </div>
      {/* <div className='bottomSpace'></div> */}
    </div>
  )
}

export default Visual
