<script lang="ts">
import {NButton} from 'naive-ui'
import {CloudUploadOutline as UploadIcon, CreateOutline as EditIcon, ImageOutline as PhotoIcon} from '@vicons/ionicons5'
import {AlbumPhotoGrid} from "#components";

export default defineComponent({
  computed: {
    AlbumPhotoGrid() {
      return AlbumPhotoGrid
    }
  },
  components: {
    UploadIcon,
    PhotoIcon
  },
  setup() {
    definePageMeta({
      key: 'album-view',
      keepalive: true,
      parent: '/',
      showFavicon: false,
      showTitle: false,
      actionComponents: [
        () => h(NButton, {
          'onClick': (e) => {
            const route = useRoute()
            const albumId = route.params['album'] as string
            return navigateTo(`${albumId}/edit`)
          },
          'quaternary': true,
          'render-icon': () => h(EditIcon)
        }, () => 'Edit album')
      ]
    })

    const route = useRoute()
    const albumId = ref(route.params['album'] as string)
    const photoId = ref(route.params['photo'] as string | undefined)

    return {albumId, photoId}
  },
  data() {
    return {
      showUpload: false
    }
  },
  methods: {
    openUpload() {
      this.showUpload = true
    },
    closeUpload() {
      this.showUpload = false
    }
  },
  watch: {
    $route(to, from) {
      const route = useRoute()
      this.albumId = route.params['album'] as string
      this.photoId = route.params['photo'] as string | undefined || undefined
    }
  }
})
</script>

<template>
  <div>
    <album-upload-modal
        v-model:open="showUpload"
        @uploadsDone="() => $refs.photoGrid.loadPhotos()"
        :album-id="albumId"
    />

    <transition>
      <div v-if="photoId">
        <photo-preview v-if="photoId" :photo-id="photoId"/>
      </div>
    </transition>

    <n-space vertical>
      <album-header :album-id="albumId">
        <n-button quaternary @click="openUpload">
          <template #icon>
            <upload-icon/>
          </template>
          Upload photos
        </n-button>
      </album-header>
      <album-photo-grid ref="photoGrid" :album-id="albumId">
        <template #empty>
          <n-empty>
            <template #icon>
              <n-icon>
                <photo-icon/>
              </n-icon>
            </template>
            Album has no photos
            <template #extra>
              <n-button @click="openUpload">Upload photos</n-button>
            </template>
          </n-empty>
        </template>
      </album-photo-grid>
    </n-space>
  </div>
</template>

<style scoped>
.v-enter-active,
.v-leave-active {
  transition: opacity ease-in-out 200ms;
  position: relative;
  z-index: 1;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}
</style>
