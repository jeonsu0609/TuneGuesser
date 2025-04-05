import React from 'react'
import './MainContent.scss'

const MainContent: React.FC = () => {
  return (
    <div className='main-content'>
      <div className='album-art'>
        <img src='https://via.placeholder.com/400' alt='Album Art' />
      </div>
      <div className='song-info'>
        <h2>Song Title</h2>
        <p>Artist Name</p>
      </div>
    </div>
  )
}

export default MainContent
