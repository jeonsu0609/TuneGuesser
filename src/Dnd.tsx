import React, { FC, useCallback, useState } from 'react'
import { useRecoilState } from 'recoil'
import type { Song } from './entity'
import { listState, songState } from './atoms'
import { invoke } from '@tauri-apps/api/tauri'
import Player from './Player'
import './Dnd.scss'

// fake data generator
const getItems = (count: number) =>
  Array.from({ length: count }, (v, k) => k).map((k) => ({
    id: `item-${k}`,
    content: `item ${k}`,
  }))

// a little function to help us with reordering the result
const reorder = (list: any, startIndex: number, endIndex: number) => {
  const result = Array.from(list)
  const [removed] = result.splice(startIndex, 1)
  result.splice(endIndex, 0, removed)

  return result
}

const grid = 8

const getItemStyle = (isDragging: boolean, draggableStyle: any) => ({
  // some basic styles to make the items look a bit nicer
  userSelect: 'none',
  padding: grid * 2,
  margin: `0 0 ${grid}px 0`,

  // change background colour if dragging
  background: isDragging ? '#17382d' : 'transparent',
  color: isDragging ? '#ffffff' : '#000000',
  border: '2px solid #ffffff',

  // styles we need to apply on draggables
  ...draggableStyle,
})

const getListStyle = (isDraggingOver: boolean) => ({
  background: isDraggingOver ? '#255b49' : '#347f66',
  padding: grid,
  width: 250,
})

export function DndExample() {
  const [items, setItems] = useRecoilState<Song[]>(listState)

  const onDragEnd = (result: any) => {
    if (!result.destination) {
      return
    }

    const _items = reorder(items, result.source.index, result.destination.index)

    setItems(_items as Song[])
  }

  const handleClick = async (item: any) => {
    const res = await invoke('simple_test', {
      argument: item.filename,
    })
    console.log(res)
  }

  return (
    <div className='app'>
      <Player />
    </div>
  )
}
