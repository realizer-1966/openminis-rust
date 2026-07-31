// 미디어 재생 — 오디오/비디오 제어
// 원본: MediaPlayerManager.kt (214줄)
// → 최소화 (~40줄)

package com.openminis.app

import android.content.Context
import android.media.MediaPlayer

class MediaPlayerManager(private val context: Context) {

    private var player: MediaPlayer? = null

    fun play(path: String) {
        player?.release()
        player = MediaPlayer().apply {
            setDataSource(path)
            prepare()
            start()
        }
    }

    fun pause() { player?.pause() }
    fun resume() { player?.start() }
    fun seekTo(ms: Int) { player?.seekTo(ms) }
    fun stop() { player?.let { it.stop(); it.release() }; player = null }

    val isPlaying: Boolean get() = player?.isPlaying ?: false
    val duration: Int get() = player?.duration ?: 0
    val position: Int get() = player?.currentPosition ?: 0
}