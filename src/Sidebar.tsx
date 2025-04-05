import React from 'react'
import './Sidebar.scss'

const Sidebar: React.FC = () => {
  return (
    <div className='sidebar'>
      <div className='logo'>Melon Player</div>
      <nav>
        <ul>
          <li>Home</li>
          <li>Browse</li>
          <li>Radio</li>
        </ul>
      </nav>
      <div className='playlists'>
        <h3>Playlists</h3>
        <ul>
          <li>Chill Hits</li>
          <li>Pop Mix</li>
          <li>Workout Beats</li>
        </ul>
      </div>
    </div>
  )
}

export default Sidebar
