module.exports = {
  devServer: {
    proxy: {
      '/ws': {
        target: 'ws://localhost:8000',
        ws: true
      }
    }
  }
}
