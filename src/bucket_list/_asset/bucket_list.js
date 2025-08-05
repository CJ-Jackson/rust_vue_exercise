import {createApp} from "vue";

createApp({
    data() {
        return {
            bucket_list: [],
            input_name: "",
            input_description: "",
            error: false,
        }
    },
    methods: {
        getBucketList() {
            fetch('/bucket_list/all')
                .then(res => res.json())
                .then(data => {
                    this.bucket_list = data;
                    this.formatDate();
                });
        },
        formatDate() {
            this.bucket_list.forEach(item => {
                item.timestamp = new Date(item.timestamp).toLocaleString();
            });
        },
        addToBucketList() {
            if (this.input_name === "" || this.input_description === "") {
                alert('Please fill in the required fields');
                return;
            }
            let json = {
                name: this.input_name,
                description: this.input_description
            }
            fetch('/bucket_list/add', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(json)
            }).then(res => {
                if (res.status === 200) {
                    this.getBucketList();
                    this.input_name = "";
                    this.input_description = "";
                    this.error = false;
                } else if (res.status === 422) {
                    let content = res.json();
                    content.then(data => {
                        let sorted = {};
                        for (let key in data) {
                            sorted[data[key].field_name] = data[key].messages;
                        }
                        this.error = sorted;
                    });
                }
            })
        }
    },
    mounted() {
        this.getBucketList()
    },
}).mount('#bucket-list');