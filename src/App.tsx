import React, { ChangeEvent, useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { emit, listen } from '@tauri-apps/api/event'
import {
  FormControl,
  InputLabel,
  NativeSelect,
  SelectChangeEvent,
  Drawer,
  Tabs,
  Tab,
  Box,
  Typography,
  IconButton,
  AppBar,
  Toolbar,
} from '@mui/material'
import MenuIcon from '@mui/icons-material/Menu'
import './App.scss'
import { DndExample } from './Dnd'
import { useRecoilState } from 'recoil'
import type { Song } from './entity'
import { isPlayingState, listState, songState } from './atoms'
import Visual from './Visual'
import { ToastContainer, toast } from 'react-toastify'
import 'react-toastify/dist/ReactToastify.css'
import { WebviewWindow } from '@tauri-apps/api/window'
import { http } from '@tauri-apps/api'
import New from './New'
import Deepseek from './Deepseek'

function App() {
  const [msgFromRust, setMsgFromRust] = useState('')
  const [inputValue, setInputValue] = useState('')
  const [blur, setBlur] = useState(true)
  const [isDndView, setIsDndView] = useState(false)
  const [items, setItems] = useRecoilState<Song[]>(listState)
  const [item, setItem] = useRecoilState<Song>(songState)
  const [year, setYear] = useState(2024)
  const audioRef = useRef<HTMLMediaElement>(null)
  const [playing, setPlaying] = useRecoilState(isPlayingState)
  const [tabIndex, setTabIndex] = useState<number>(0)
  const [drawerOpen, setDrawerOpen] = useState<boolean>(false)

  const YEAR_SELECT = Array.from(
    { length: 2024 - 2000 + 1 },
    (value, index) => 2000 + index
  )

  useEffect(() => {
    invoke('start_server')
    invoke('listen')

    //listen to a event
    const unlisten = listen('rs2js', (e) => {
      console.log(e)
    })

    const login = listen('login', (e) => {
      invoke('start_server')
    })

    const correct = listen('correct', (e) => {
      toast('Wow so easy!')
      setBlur(false)
    })

    const song = listen('song', (e) => {
      // console.log(e);
      setBlur(true)
      setItem(e.payload as Song)
    })

    const state = listen('state', (e) => {
      // console.log(e);
      setPlaying(e.payload as boolean)
    })

    // emits the `click` event with the object payload
    emit('event-name', {
      theMessage: 'Tauri is awesome!',
    })

    return () => {
      unlisten.then((f) => f())
      song.then((f) => f())
      state.then((f) => f())
      login.then((f) => f())
    }
  }, [])

  const togglePlay = () => {
    invoke('play', {})
  }
  const toggleTest = async () => {
    const res = await invoke('simple_command_with_result', {
      argument: year.toString(),
    })
    setItems(res as Song[])
  }

  const toggle_hls = () => {
    invoke('toggle_hls', {})
  }

  const toggleTest2 = async () => {
    const res = await invoke('simple_test', {
      argument: './assets/NewJeans-01-Attention.mp3',
    })
  }

  const toggleTest3 = async () => {
    const res = await invoke('read_files', {})
    setItems(res as Song[])
  }

  const handleChange = (e: ChangeEvent<HTMLSelectElement>) => {
    setYear(Number(e.target.value))
  }

  const handleHelloWorld = async () => {
    try {
      const response = await invoke('hello_world_test', {
        event: inputValue || 'nope',
      })
      setMsgFromRust(`${response}`)
      console.log('response ', response)
    } catch (error) {
      console.log('error ', error)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleHelloWorld() // 작성한 댓글 post 요청하는 함수
    }
  }

  const handleNextBtnClick = () => {
    invoke('next', {})
  }

  const MelonAPI = {
    login: (info: any) => {
      window.close()
    },
  }

  const handleOpenPopup = async () => {
    invoke('open_login')
  }

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setTabIndex(newValue)
  }

  const toggleDrawer = () => {
    setDrawerOpen(!drawerOpen)
  }

  return (
    <div className='app'>
      <div className='btn-toggle-container'>
        <IconButton
          color='inherit'
          aria-label='open drawer'
          onClick={toggleDrawer}
          edge='start'
          sx={{ width: 60, height: 40 }}
        >
          <MenuIcon />
        </IconButton>
        <button
          className='btn-toggle-setting'
          aria-label='toggle view'
          onClick={toggle_hls}
        >
          ⚙️
        </button>
      </div>
      <Drawer
        variant='temporary'
        anchor='left'
        open={drawerOpen}
        onClose={toggleDrawer}
        sx={{
          flexShrink: 0,
          [`& .MuiDrawer-paper`]: { boxSizing: 'border-box' },
        }}
      >
        <Tabs
          orientation='vertical'
          value={tabIndex}
          onChange={handleTabChange}
          aria-label='Vertical tabs example'
          sx={{ borderRight: 1, borderColor: 'divider' }}
        >
          <Tab label='Home' />
          <Tab label='Dnd Example' />
          <Tab label='Visual' />
          <Tab label='Copilot' />
          <Tab label='Deepseek' />
        </Tabs>
      </Drawer>
      <TabPanel value={tabIndex} index={0}>
        <header className='App-header'>
          <FormControl>
            <InputLabel variant='standard' htmlFor='uncontrolled-native'>
              Year
            </InputLabel>
            <NativeSelect defaultValue={year} onChange={handleChange}>
              {YEAR_SELECT.map((year, idx) => {
                return (
                  <option key={idx} value={year}>
                    {year}년
                  </option>
                )
              })}
            </NativeSelect>
          </FormControl>
          <div className='component-wrapper'>
            <input
              value={inputValue}
              placeholder='input for rust'
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={handleKeyDown}
            />
          </div>
          <div>
            <img
              style={{
                filter: blur ? 'blur(30px)' : 'none',
                transition: blur ? 'none' : 'filter 0.3s ease-out',
              }}
              className='album-img'
              src={item?.img}
              alt='AlbumImage'
            />
          </div>
          <div>
            <button className='btn' onClick={toggleTest}>
              Start
            </button>
            <button className='btn' onClick={handleNextBtnClick}>
              Next
            </button>
            {/* <button className='btn' onClick={handleOpenPopup}>
              toggleTest
            </button>
            <button className='btn' onClick={toggleTest2}>
              File
            </button> */}
            <audio ref={audioRef} />
          </div>
          <div>
            <ToastContainer
              position='bottom-center' // 알람 위치 지정
              autoClose={3000} // 자동 off 시간
              hideProgressBar={false} // 진행시간바 숨김
              closeOnClick // 클릭으로 알람 닫기
              rtl={false} // 알림 좌우 반전
              pauseOnFocusLoss // 화면을 벗어나면 알람 정지
              draggable // 드래그 가능
              pauseOnHover // 마우스를 올리면 알람 정지
              theme='light'
              limit={1} // 알람 개수 제한
            />
          </div>
        </header>
      </TabPanel>
      <TabPanel value={tabIndex} index={1}>
        <DndExample />
      </TabPanel>
      <TabPanel value={tabIndex} index={2}>
        <Visual />
      </TabPanel>
      <TabPanel value={tabIndex} index={3}>
        <New />
      </TabPanel>
      <TabPanel value={tabIndex} index={4}>
        <Deepseek />
      </TabPanel>
    </div>
  )
}

interface TabPanelProps {
  children?: React.ReactNode
  index: number
  value: number
}

const TabPanel = (props: TabPanelProps) => {
  const { children, value, index, ...other } = props

  return (
    <div
      role='tabpanel'
      hidden={value !== index}
      id={`vertical-tabpanel-${index}`}
      aria-labelledby={`vertical-tab-${index}`}
      {...other}
    >
      {value === index && (
        <Box p={3}>
          <Typography>{children}</Typography>
        </Box>
      )}
    </div>
  )
}

export default App
