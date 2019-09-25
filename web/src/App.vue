<template>
  <div id="app">
    <section class="hero">
      <div class="hero-body">
        <div class="container">
          <h1 class="title">ColorUI</h1>
          <h2 class="subtitle">Control interface</h2>
        </div>
      </div>
    </section>

    <section class="section">
      <div class="columns">
        <div class="column" :key="led.id" v-for="led in leds">
          <div class="picker">
            <h3 class="is-size-4">{{ led.id + 1 }} Instant color</h3>
            <picker @input="colorChanged(led.id)" v-model="led.color"></picker>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script>
import Chrome from 'vue-color/src/components/Chrome.vue'
import convert from 'color-convert'

function newLed (index) {
  return {
    id: index,
    color: '#000000'
  }
}

function rgbStruct (color) {
  let rgb = convert.hex.rgb(typeof color.hex !== 'undefined' ? color.hex : color)
  return {
    r: rgb[0],
    g: rgb[1],
    b: rgb[2]
  }
}

export default {
  name: 'App',
  components: {
    'picker': Chrome
  },
  data: function () {
    return {
      leds: [
        newLed(0),
        newLed(1),
        newLed(2)
      ]
    }
  },
  methods: {
    colorChanged (id) {
      localStorage.data = JSON.stringify({ leds: this.leds })

      this.$socket.sendObj({
        led: 1 << id,
        ...rgbStruct(this.leds[id].color)
      })
    },
    sendAllLeds () {
      for (var index = 0; index < this.leds.length; ++index) {
        this.colorChanged(index)
      }
    }
  },
  mounted () {
    try {
      var data = JSON.parse(localStorage.data)
      if (Array.isArray(data.leds)) {
        this.leds = data.leds
      }
    } catch (error) {
    }

    setTimeout(this.sendAllLeds, 100)
  }
}
</script>

<style>
#app {
  font-family: "Avenir", Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}
.picker {
  margin: 0 auto;
  width: 225px;
}
</style>
