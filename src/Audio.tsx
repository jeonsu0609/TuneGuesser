import React, { useEffect, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import Hls from 'hls.js'
import { useRecoilState, useSetRecoilState } from 'recoil'
import { isPlayingState, volumeState, progressState, seekState } from './atoms'
import { invoke } from '@tauri-apps/api'

function Audio() {
  const audioRef = useRef<HTMLMediaElement>(null)
  const [volume, setVolume] = useRecoilState(volumeState)
  const [playing, setPlaying] = useRecoilState(isPlayingState)
  const [progressBar, setProgressBar] = useRecoilState(progressState)
  const [seek, setSeek] = useRecoilState(seekState)
  const sourceBufferRef = useRef<SourceBuffer | null>(null)
  const mediaSourceRef = useRef<MediaSource | null>(null)
  const [audioChunks, setAudioChunks] = useState<Uint8Array[]>([])
  const [isLastChunk, setIsLastChunk] = useState(false)

  const handleEnded = () => {
    invoke('next', {})
    setIsLastChunk(false)
  }

  const handleTimeUpdate = () => {
    if (audioRef.current && isLastChunk) {
      const buffered = audioRef.current.buffered
      setProgressBar(audioRef.current.currentTime)
      console.log(
        audioRef.current.currentTime,
        Math.floor(buffered.end(buffered.length - 1))
      )
      if (
        buffered.length > 0 &&
        audioRef.current.currentTime >=
          Math.floor(buffered.end(buffered.length - 1))
      ) {
        handleEnded()
      }
    }
  }

  useEffect(() => {
    if (audioRef.current) {
      audioRef.current.currentTime = seek
    }
  }, [seek])

  useEffect(() => {
    audioRef.current?.addEventListener('timeupdate', handleTimeUpdate)
    return () => {
      audioRef.current?.removeEventListener('timeupdate', handleTimeUpdate)
    }
  }, [isLastChunk])

  useEffect(() => {
    if (!audioRef) {
      return
    }

    playing ? audioRef.current?.play() : audioRef.current?.pause()
    invoke('play_or_pause', { playing })
  }, [playing])

  useEffect(() => {
    if (audioRef && audioRef.current && volume) {
      audioRef.current.volume = volume / 100
    }
  }, [volume])

  useEffect(() => {
    const hls = listen('hls', (e) => {
      const hls = new Hls()
      hls.loadSource(e.payload as string)
      if (audioRef.current) hls.attachMedia(audioRef.current)
      audioRef.current?.play()
    })

    const init = listen('chunk_start', (e) => {
      if (mediaSourceRef.current) {
        mediaSourceRef.current.endOfStream()
      }
      mediaSourceRef.current = new MediaSource()
      if (audioRef.current) {
        audioRef.current.src = URL.createObjectURL(mediaSourceRef.current)
      }
      mediaSourceRef.current.addEventListener('sourceopen', () => {
        if (mediaSourceRef.current) {
          sourceBufferRef.current = mediaSourceRef.current.addSourceBuffer(
            'audio/mp4; codecs="mp4a.40.2"'
          )
          sourceBufferRef.current.addEventListener('updateend', () => {
            if (
              audioChunks.length > 0 &&
              sourceBufferRef.current &&
              !sourceBufferRef.current.updating
            ) {
              const chunk = audioChunks.shift()
              if (chunk) {
                sourceBufferRef.current.appendBuffer(chunk)
              }
            } else if (
              isLastChunk &&
              sourceBufferRef.current &&
              !sourceBufferRef.current.updating
            ) {
              if (mediaSourceRef.current) {
                mediaSourceRef.current.endOfStream()
              }
            }
          })
        }
      })
    })

    const next = listen('next', (e) => {
      handleEnded()
    })

    const end = listen('chunk_end', (e) => {
      audioRef.current?.play()
      setIsLastChunk(true)
    })

    const audio = listen<string>('decrypted_audio', (event) => {
      const base64Audio = event.payload
      const audioUrl = `data:audio/mp4;base64,${base64Audio}`
      audioRef.current?.setAttribute('src', audioUrl)
      audioRef.current?.play()
    })

    const audioChunk = listen<string>('decrypted_audio_chunk', (event) => {
      const audioData = new Uint8Array(event.payload as any)
      if (
        audioChunks.length < 1 &&
        sourceBufferRef.current &&
        !sourceBufferRef.current.updating
      ) {
        sourceBufferRef.current.appendBuffer(audioData)
      } else {
        setAudioChunks((prevChunks) => [...prevChunks, audioData])
      }
    })

    return () => {
      hls.then((f) => f())
      audio.then((fn) => fn())
      audioChunk.then((fn) => fn())
      init.then((fn) => fn())
      end.then((fn) => fn())
      next.then((fn) => fn())
    }
  }, [])

  return <audio id='audio-player' ref={audioRef} onEnded={handleEnded} />
}

export default Audio
