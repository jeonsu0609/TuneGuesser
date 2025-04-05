import { atom } from 'recoil'

export const songState = atom({
  key: 'songState',
  default: {
    id: '',
    title: '',
    artist: '',
    album: '',
    filename: '',
    img: '',
    duration: '',
  },
})

export const listState = atom({
  key: 'listState',
  default: [
    {
      id: '',
      title: '',
      artist: '',
      album: '',
      filename: '',
      img: '',
      duration: '',
    },
  ],
})

export const songTitleState = atom({
  key: 'songTitleState',
  default: 'empty',
})

export const progressState = atom({
  key: 'progressState',
  default: 0,
})

export const seekState = atom({
  key: 'seekState',
  default: 0,
})

export const songIdState = atom({
  key: 'songIdState',
  default: '',
})

export const isPlayingState = atom({
  key: 'isPlayingState',
  default: false,
})

export const volumeState = atom({
  key: 'volumeState',
  default: 30,
})
