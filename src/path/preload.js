function addOrUpdateUrlParam(name, value) {
  var href = window.location.href
  var regex = new RegExp('[&\\?]' + name + '=')
  if (regex.test(href)) {
    regex = new RegExp('([&\\?])' + name + '=\\d+')
    window.location.href = href.replace(regex, '$1' + name + '=' + value)
  } else {
    if (href.indexOf('?') > -1)
      window.location.href = href + '&' + name + '=' + value
    else window.location.href = href + '?' + name + '=' + value
  }
}

const MelonAPI = {
  login: (info) => {
    document.dispatchEvent(new CustomEvent('login'), { info })
    window.dispatchEvent(new CustomEvent('login'), { info })

    let param = ''
    for (const [key, value] of Object.entries(info)) {
      if (!['sessionId', 'token', 'memberKey'].includes(key)) {
        continue
      }

      if (param) {
        param = param + '&' + key + '=' + value
      } else {
        param = '?' + key + '=' + value
      }
    }

    document.location.href = 'http://localhost:12345' + param
  },
}
