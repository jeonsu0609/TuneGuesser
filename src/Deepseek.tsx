import React from 'react'
import Sidebar from './Sidebar'
import MainContent from './MainContent'
import Footer from './Footer'
import './Deepseek.scss'

const Deepseek: React.FC = () => {
  return (
    <div>
      <Sidebar />
      <MainContent />
      <Footer />
    </div>
  )
}

export default Deepseek
