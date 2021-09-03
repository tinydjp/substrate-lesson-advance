import React, { useEffect, useState } from 'react'
import { Form, Grid } from 'semantic-ui-react'

import { useSubstrate } from './substrate-lib'
import { TxButton } from './substrate-lib/components'

import KittyCards from './KittyCards'

export default function Kitties (props) {
  const { api, keyring } = useSubstrate()
  const { accountPair } = props

  const [kitties, setKitties] = useState([])
  const [status, setStatus] = useState('')
  const [total, setTotal] = useState(0)

  const fetchKitties = () => {
    // TODO: 在这里调用 `api.query.kittiesModule.*` 函数去取得猫咪的信息。
    api.query.kittiesModule.kittiesCount(
        c => {
            c && setTotal(parseInt(c))
        }
    ).catch(console.error)
  }

  const populateKitties = () => {
    // TODO: 在这里添加额外的逻辑。你需要组成这样的数组结构：
    let kitties = []
    const indice = [...Array(total).keys()];
    const ownerQuery = api.query.kittiesModule.owner.multi(indice)
    const kittiesQuery = api.query.kittiesModule.kitties.multi(indice)
    Promise.all([ownerQuery, kittiesQuery]).then(
        r => {
            const [_owners, _kitties] = r
            for(let i = 0; i<_owners.length; i++) {
                kitties.push(
                    {
                        id: i,
                        dna: _kitties[i].unwrap(),
                        owner: _owners[i].unwrap().toString()
                    }
                )
            }
            setKitties(kitties)
        }
    )
  }

  useEffect(fetchKitties, [api, keyring])
  useEffect(populateKitties, [total])

  return <Grid.Column width={16}>
    <h1>小毛孩</h1>
    <KittyCards kitties={kitties} accountPair={accountPair} setStatus={setStatus}/>
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
          accountPair={accountPair} label='创建小毛孩' type='SIGNED-TX' setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'create',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>
}
