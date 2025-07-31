import {createApp, ref} from "vue";

createApp({
    data() {
        const message = ref('Hello vue!')
        return {
            message
        }
    }
}).mount('#app');

createApp({
    data() {
        return {
            count: ref(0)
        }
    }
}).mount('#counter');

createApp({
    data() {
        return {
            items: []
        }
    },
    methods: {
        getItems() {
            fetch('/array')
                .then(res => res.json())
                .then(data => this.items = data);
        }
    },
    mounted() {
        this.getItems()
    }
}).mount('#array');