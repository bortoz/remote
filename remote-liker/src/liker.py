import sys, json, re
from http.cookiejar import CookieJar, Cookie
from requests import Session
from requests.adapters import HTTPAdapter
from urllib.parse import urlparse, parse_qs
from urllib3.util.retry import Retry


class OlinfoLiker:

    def __init__(self, target, token):
        self.__session = Session()
        retry = Retry(backoff_factor=.1, status_forcelist=[429])
        adapter = HTTPAdapter(max_retries=retry)
        self.__session.mount('https://', adapter)
        self.__buffer = []
        self.__lastID = 0
        self.__login(token)
        self.__setCSRF()
        self.__target = target

    def like(self):
        post = self.__getPost()
        url = 'https://forum.olinfo.it/post_actions'
        data = {'id': post['id'], 'post_action_type_id': 2}
        headers = {
            'X-CSRF-Token': self.__csrf,
            'X-Requested-With': 'XMLHttpRequest'
        }
        r = self.__session.post(url, headers=headers, json=data)
        data = self.__parseJSON(r.text)
        return post['username'], post['id']

    def __login(self, token):
        self.__session.cookies.set_cookie(
            Cookie(0, 'token', token, None, False, '.olinfo.it', '.olinfo.it',
                   '.olinfo.it', '/', True, None, None, False, None, None, {}))
        url = 'https://training.olinfo.it/api/user'
        data = {'action': 'me'}
        r = self.__session.post(url, json=data)
        try:
            data = self.__parseJSON(r.text)
        except:
            raise Exception('login fallito')

        url = 'https://forum.olinfo.it/session/sso?return_path=%2F'
        r = self.__session.get(url)
        query = parse_qs(urlparse(urlparse(r.url).fragment).query)

        url = 'https://training.olinfo.it/api/sso'
        payload = {'payload': query['sso'][0], 'sig': query['sig'][0]}
        r = self.__session.post(url, json=payload)
        data = self.__parseJSON(r.text)

        url = 'https://forum.olinfo.it/session/sso_login?' + data['parameters']
        r = self.__session.get(url)

        self.__username = r.headers['X-Discourse-Username']

    def __parseJSON(self, text):
        try:
            data = json.loads(text)
        except json.decoder.JSONDecodeError:
            raise Exception(re.search('<title>(.*?)</title>', text).group(1))
        if 'error' in data:
            raise Exception(data['error'])
        if 'errors' in data:
            for e in data['errors']:
                raise Exception(e)
        return data

    def __setCSRF(self):
        url = 'https://forum.olinfo.it/session/csrf.json'
        r = self.__session.get(url)
        data = self.__parseJSON(r.text)
        self.__csrf = data['csrf']

    def __getPostsTarget(self):
        url = 'https://forum.olinfo.it/user_actions.json?offset=%d&username=%s&filter=5' % (
            self.__lastID, self.__target)
        r = self.__session.get(url)
        self.__lastID += 30
        try:
            data = self.__parseJSON(r.text)
        except:
            raise Exception('Utente non trovato')
        if not data['user_actions']:
            raise Exception('Nessun post trovato')
        ids = [i['post_id'] for i in data['user_actions']]
        while ids:
            url = 'https://forum.olinfo.it/posts.json?before=%d' % ids[0]
            r = self.__session.get(url)
            data = self.__parseJSON(r.text)
            for i in data['latest_posts']:
                if i['id'] not in ids:
                    continue
                ids.remove(i['id'])
                for j in i['actions_summary']:
                    if 'acted' in j:
                        break
                else:
                    self.__buffer.append(i)

    def __getPost(self):
        while not self.__buffer:
            self.__getPostsTarget()
        return self.__buffer.pop()


def run(user, token):
    liker = OlinfoLiker(user, token)
    try:
        like_count = 0
        while True:
            ret = liker.like()
            like_count += 1
    except Exception as e:
        return ('Like ricevuti: %d' % like_count, str(e))
