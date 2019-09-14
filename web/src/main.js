import Vue from 'vue'
import App from './App.vue'
import Buefy from 'buefy'
import 'buefy/dist/buefy.css'

import VueNativeSock from 'vue-native-websocket'

Vue.use(Buefy)
Vue.use(VueNativeSock, 'ws://' + window.location.host + '/ws/', { format: 'json',
  reconnection: true,
  reconnectionAttempts: 5,
  reconnectionDelay: 3000
})

Vue.config.productionTip = false

new Vue({
  render: h => h(App)
}).$mount('#app')
